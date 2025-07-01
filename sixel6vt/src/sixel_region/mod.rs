use std::collections::{HashMap, HashSet, BTreeMap};
use std::mem;
use thiserror::Error;

/// The maximum number of patterns stored in the PatternDictionary.
const MAX_PATTERN_COUNT: usize = 256;
/// Each sixel data byte encodes up to SIXEL_HEIGHT (6) vertical pixels.
const SIXEL_HEIGHT: usize = 6;
/// Default capacity for region-based data structures.
const DEFAULT_REGION_CAPACITY: usize = 64;
/// Default capacity hint for newly allocated lines or child nodes.
const DEFAULT_LINE_CAPACITY: usize = 64;
/// Base grid cell size for the spatial index.
const BASE_GRID_CELL_SIZE: usize = 12;

/// A single pixel in the final raster image.
#[derive(Debug, Clone)]
pub struct Pixel {
    pub on: bool,
    pub color: u16,
}

/// Represents a color with 8-bit RGB components.
#[derive(Debug, Clone)]
pub struct SixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl SixelColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Final raster representation of a Sixel image.
#[derive(Debug)]
pub struct SixelImage {
    /// 2D buffer of pixels, indexed by [y][x].
    pub pixels: Vec<Vec<Pixel>>,
    /// Mapping from color register index to SixelColor.
    pub color_registers: BTreeMap<u16, SixelColor>,
}

impl SixelImage {
    /// Create an empty image with no pixels and no color registers.
    pub fn new() -> Self {
        Self {
            pixels: Vec::new(),
            color_registers: BTreeMap::new(),
        }
    }

    /// Returns the width of the image (0 if empty).
    pub fn width(&self) -> usize {
        if self.pixels.is_empty() {
            0
        } else {
            self.pixels[0].len()
        }
    }

    /// Returns the height of the image.
    pub fn height(&self) -> usize {
        self.pixels.len()
    }
}

/// Error variants for this Sixel decoder.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SixelError {
    #[error("Missing DCS (Device Control String) start sequence")]
    MissingDcs,
    #[error("Encountered invalid sixel sequence")]
    InvalidSequence,
    #[error("Image width exceeds maximum")]
    WidthOverflow,
    #[error("Image height exceeds maximum")]
    HeightOverflow,
    #[error("Invalid sixel data byte (< 63)")]
    InvalidDataByte,
    #[error("Dictionary error: {0}")]
    DictionaryError(String),
}

/// Minimal subset of Sixel events. Usually, real decoders might parse them from raw byte streams.
#[derive(Debug, Clone)]
pub enum SixelEvent {
    /// Start of the Sixel data. transparent_background can be 0 or 1.
    Dcs { transparent_background: Option<u8> },
    /// Introduce a color or select an existing color number.
    ColorIntroducer { color_number: u16, r: Option<u8>, g: Option<u8>, b: Option<u8> },
    /// A single data byte with optional repeat.
    Data { byte: u8 },
    /// A run-length data byte (repeat).
    Repeat { repeat_count: usize, byte_to_repeat: u8 },
    /// Move cursor to the beginning of line (x=0).
    GotoBeginningOfLine,
    /// Move cursor down by SIXEL_HEIGHT pixels and x=0.
    GotoNextLine,
    /// Raster attributes for scaling or bounding box.
    RasterAttribute { ph: Option<usize>, pv: Option<usize> },
    /// Unknown or invalid sequence.
    UnknownSequence(Vec<u8>),
    /// End of the sixel data stream.
    End,
}

/// A pattern dictionary for storing up to 256 unique Sixel patterns (bytes 63..=126).
#[derive(Debug, Default)]
struct PatternDictionary {
    patterns: HashSet<u8>,
    frequency: HashMap<u8, usize>,
}

impl PatternDictionary {
    fn new() -> Self {
        Self {
            patterns: HashSet::with_capacity(MAX_PATTERN_COUNT),
            frequency: HashMap::with_capacity(MAX_PATTERN_COUNT),
        }
    }

    /// Insert a pattern byte if not present. pattern must be >= 63.
    fn get_or_insert(&mut self, pattern: u8) -> Result<u64, SixelError> {
        if pattern < 63 {
            return Err(SixelError::InvalidDataByte);
        }
        if self.patterns.len() >= MAX_PATTERN_COUNT {
            return Err(SixelError::DictionaryError("Pattern dictionary full".to_string()));
        }
        self.patterns.insert(pattern);
        *self.frequency.entry(pattern).or_insert(0) += 1;
        Ok(pattern as u64)
    }

    /// Check whether a given bit is set in (pattern - 63).
    fn is_bit_set(&self, pattern: u8, bit: usize) -> bool {
        (pattern.saturating_sub(63)) & (1 << bit) != 0
    }
}

/// A quad tree for subdividing large uniform or pattern-based areas. Not strictly required, but can help hierarchical queries.
#[derive(Debug, Clone)]
enum QuadNode {
    Leaf {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        color: u16,
        pattern_hash: Option<u64>,
    },
    Branch {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        children: [Option<Box<QuadNode>>; 4],
    },
}

impl QuadNode {
    fn new_leaf(x: usize, y: usize, width: usize, height: usize, color: u16, pattern_hash: Option<u64>) -> Self {
        QuadNode::Leaf { x, y, width, height, color, pattern_hash }
    }

    fn new_branch(x: usize, y: usize, width: usize, height: usize) -> Self {
        QuadNode::Branch {
            x, y, width, height,
            children: [None, None, None, None],
        }
    }

    fn add_child(&mut self, child: QuadNode) -> Result<(), SixelError> {
        if let QuadNode::Branch { children, .. } = self {
            for slot in children.iter_mut() {
                if slot.is_none() {
                    *slot = Some(Box::new(child));
                    return Ok(());
                }
            }
            Err(SixelError::DictionaryError("QuadNode branch full".to_string()))
        } else {
            Err(SixelError::DictionaryError("Cannot add child to Leaf".to_string()))
        }
    }

    fn split(&mut self, min_size: usize) -> Result<(), SixelError> {
        if let QuadNode::Leaf { x, y, width, height, color, pattern_hash } = *self {
            if width > min_size && height > min_size {
                let mut branch = QuadNode::new_branch(x, y, width, height);
                let half_w = width / 2;
                let half_h = height / 2;
                branch.add_child(QuadNode::new_leaf(x, y, half_w, half_h, color, pattern_hash))?;
                branch.add_child(QuadNode::new_leaf(x + half_w, y, width - half_w, half_h, color, pattern_hash))?;
                branch.add_child(QuadNode::new_leaf(x, y + half_h, half_w, height - half_h, color, pattern_hash))?;
                branch.add_child(QuadNode::new_leaf(x + half_w, y + half_h, width - half_w, height - half_h, color, pattern_hash))?;
                *self = branch;
            }
        }
        Ok(())
    }
}

/// A rectangular region describing a block of uniform or pattern-based pixels.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Region {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: u16,
    pattern_hash: Option<u64>,
    is_solid: bool,
}

impl Region {
    fn new(x: usize, y: usize, width: usize, height: usize, color: u16, pattern_hash: Option<u64>, is_solid: bool) -> Self {
        Self {
            x,
            y,
            width,
            height,
            color,
            pattern_hash,
            is_solid,
        }
    }

    /// Check if two regions can be merged vertically (same x, width, color, pattern, etc.).
    fn can_merge_vertical(&self, other: &Self) -> bool {
        self.x == other.x
            && self.width == other.width
            && self.color == other.color
            && self.pattern_hash == other.pattern_hash
            && self.is_solid == other.is_solid
            && (self.y + self.height == other.y || other.y + other.height == self.y)
    }

    /// Check if two regions can be merged horizontally (same y, height, color, pattern, etc.).
    fn can_merge_horizontal(&self, other: &Self) -> bool {
        self.y == other.y
            && self.height == other.height
            && self.color == other.color
            && self.pattern_hash == other.pattern_hash
            && self.is_solid == other.is_solid
            && (self.x + self.width == other.x || other.x + other.width == self.x)
    }

    fn merge_vertical(&mut self, other: &Self) {
        if self.can_merge_vertical(other) {
            if self.y > other.y {
                let new_height = self.y + self.height - other.y;
                self.y = other.y;
                self.height = new_height;
            } else {
                self.height = other.y + other.height - self.y;
            }
        }
    }

    fn merge_horizontal(&mut self, other: &Self) {
        if self.can_merge_horizontal(other) {
            if self.x > other.x {
                let new_width = self.x + self.width - other.x;
                self.x = other.x;
                self.width = new_width;
            } else {
                self.width = other.x + other.width - self.x;
            }
        }
    }
}

/// A coarse grid-based spatial index for region merges.
#[derive(Debug)]
struct SpatialIndex {
    grid: HashMap<(usize, usize), Vec<Region>>,
    cell_size: usize,
}

impl SpatialIndex {
    fn new(cell_size: usize) -> Self {
        Self {
            grid: HashMap::new(),
            cell_size,
        }
    }

    fn insert(&mut self, region: Region) {
        let x_cell = region.x / self.cell_size;
        let y_cell = region.y / self.cell_size;
        self.grid.entry((x_cell, y_cell)).or_insert_with(Vec::new).push(region);
    }

    fn remove(&mut self, region: &Region) {
        let x_cell = region.x / self.cell_size;
        let y_cell = region.y / self.cell_size;
        if let Some(vec) = self.grid.get_mut(&(x_cell, y_cell)) {
            vec.retain(|r| r != region);
            if vec.is_empty() {
                self.grid.remove(&(x_cell, y_cell));
            }
        }
    }

    fn nearby_regions(&self, region: &Region) -> Vec<Region> {
        let x_cell = region.x / self.cell_size;
        let y_cell = region.y / self.cell_size;
        let mut results = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                let cx = (x_cell as isize + dx) as usize;
                let cy = (y_cell as isize + dy) as usize;
                if let Some(list) = self.grid.get(&(cx, cy)) {
                    for r in list {
                        results.push(r.clone());
                    }
                }
            }
        }
        results
    }
}

/// The main region-based Sixel decoder.
#[derive(Debug)]
pub struct SixelDecoder {
    color_registers: BTreeMap<u16, SixelColor>,
    current_color: u16,
    cursor_x: usize,
    cursor_y: usize,
    regions: HashSet<Region>,
    spatial_index: SpatialIndex,
    quadtree: Option<QuadNode>,
    pattern_dict: PatternDictionary,
    max_width: Option<usize>,
    max_height: Option<usize>,
    got_dcs: bool,
    transparent_background: bool,
    grid_cell_size: usize,
}

impl SixelDecoder {
    pub fn new() -> Self {
        let base = BASE_GRID_CELL_SIZE;
        Self {
            color_registers: BTreeMap::new(),
            current_color: 0,
            cursor_x: 0,
            cursor_y: 0,
            regions: HashSet::with_capacity(DEFAULT_REGION_CAPACITY),
            spatial_index: SpatialIndex::new(base),
            quadtree: None,
            pattern_dict: PatternDictionary::new(),
            max_width: None,
            max_height: None,
            got_dcs: false,
            transparent_background: false,
            grid_cell_size: base,
        }
    }

    /// Set max width for safety.
    pub fn set_max_width(&mut self, width: usize) {
        self.max_width = Some(width);
        self.grid_cell_size = self.optimize_grid_cell_size();
        self.spatial_index = SpatialIndex::new(self.grid_cell_size);
    }

    /// Set max height for safety.
    pub fn set_max_height(&mut self, height: usize) {
        self.max_height = Some(height);
        self.grid_cell_size = self.optimize_grid_cell_size();
        self.spatial_index = SpatialIndex::new(self.grid_cell_size);
    }

    fn optimize_grid_cell_size(&self) -> usize {
        let base = BASE_GRID_CELL_SIZE;
        if let (Some(mw), Some(mh)) = (self.max_width, self.max_height) {
            let cx = (mw + base - 1) / base;
            let _cy = (mh + base - 1) / base;
            let ideal = (mw / cx.max(1)).max(SIXEL_HEIGHT);
            ((ideal + SIXEL_HEIGHT - 1) / SIXEL_HEIGHT) * SIXEL_HEIGHT
        } else {
            base
        }
    }

    /// Process a single SixelEvent.
    pub fn handle_event(&mut self, event: SixelEvent) -> Result<(), SixelError> {
        if !self.got_dcs && !matches!(event, SixelEvent::Dcs { .. }) {
            return Err(SixelError::MissingDcs);
        }
        match event {
            SixelEvent::Dcs { transparent_background } => {
                self.got_dcs = true;
                self.transparent_background = transparent_background == Some(1);
            }
            SixelEvent::ColorIntroducer { color_number, r, g, b } => {
                if let (Some(rr), Some(gg), Some(bb)) = (r, g, b) {
                    self.color_registers.insert(color_number, SixelColor::new(rr, gg, bb));
                }
                self.current_color = color_number;
            }
            SixelEvent::Data { byte } => {
                if !self.transparent_background {
                    self.process_data_byte(byte, 1)?;
                }
                self.cursor_x += 1;
            }
            SixelEvent::Repeat { repeat_count, byte_to_repeat } => {
                if !self.transparent_background {
                    self.process_data_byte(byte_to_repeat, repeat_count)?;
                }
                self.cursor_x += repeat_count;
            }
            SixelEvent::GotoBeginningOfLine => {
                self.cursor_x = 0;
            }
            SixelEvent::GotoNextLine => {
                self.cursor_y = self.cursor_y.checked_add(SIXEL_HEIGHT).ok_or(SixelError::HeightOverflow)?;
                if let Some(mh) = self.max_height {
                    if self.cursor_y >= mh {
                        return Err(SixelError::HeightOverflow);
                    }
                }
                self.cursor_x = 0;
            }
            SixelEvent::RasterAttribute { ph, pv, .. } => {
                if !self.transparent_background {
                    if let Some(pv) = pv {
                        self.pad_vertically(pv)?;
                    }
                    if let Some(ph) = ph {
                        self.pad_horizontally(ph)?;
                    }
                }
            }
            SixelEvent::UnknownSequence(_) => {
                return Err(SixelError::InvalidSequence);
            }
            SixelEvent::End => {
                self.rebuild_quadtree();
            }
        }
        Ok(())
    }

    fn process_data_byte(&mut self, byte: u8, repeat_count: usize) -> Result<(), SixelError> {
        let bits = byte.checked_sub(63).ok_or(SixelError::InvalidDataByte)?;
        if bits == 0 {
            return Ok(());
        }
        if let Some(mw) = self.max_width {
            if self.cursor_x + repeat_count > mw {
                return Err(SixelError::WidthOverflow);
            }
        }
        let pattern_hash = if bits == 0b111111 { None } else {
            Some(self.pattern_dict.get_or_insert(byte)?)
        };
        let is_solid = bits == 0b111111;
        let region = Region::new(
            self.cursor_x,
            self.cursor_y,
            repeat_count,
            SIXEL_HEIGHT,
            self.current_color,
            pattern_hash,
            is_solid
        );
        self.try_merge_region(region);
        Ok(())
    }

    fn try_merge_region(&mut self, mut region: Region) {
        let nearby = self.spatial_index.nearby_regions(&region);
        let mut to_remove = Vec::new();

        for existing in &nearby {
            if region.can_merge_vertical(existing) {
                region.merge_vertical(existing);
                to_remove.push(existing.clone());
            }
        }
        if to_remove.is_empty() {
            for existing in &nearby {
                if region.can_merge_horizontal(existing) {
                    region.merge_horizontal(existing);
                    to_remove.push(existing.clone());
                }
            }
        }
        for r in &to_remove {
            self.regions.remove(r);
            self.spatial_index.remove(r);
        }
        self.regions.insert(region.clone());
        self.spatial_index.insert(region);
    }

    fn pad_vertically(&mut self, height: usize) -> Result<(), SixelError> {
        if let Some(mh) = self.max_height {
            if height > mh {
                return Err(SixelError::HeightOverflow);
            }
        }
        let region = Region::new(
            0,
            self.cursor_y,
            self.cursor_x,
            height,
            self.current_color,
            None,
            true
        );
        self.try_merge_region(region);
        Ok(())
    }

    fn pad_horizontally(&mut self, width: usize) -> Result<(), SixelError> {
        if let Some(mw) = self.max_width {
            if width > mw {
                return Err(SixelError::WidthOverflow);
            }
        }
        if width > self.cursor_x {
            let region = Region::new(
                self.cursor_x,
                self.cursor_y,
                width - self.cursor_x,
                SIXEL_HEIGHT,
                self.current_color,
                None,
                true
            );
            self.try_merge_region(region);
        }
        Ok(())
    }

    fn rebuild_quadtree(&mut self) {
        if self.regions.is_empty() {
            self.quadtree = None;
            return;
        }
        let mut max_x = 0;
        let mut max_y = 0;
        for region in &self.regions {
            max_x = max_x.max(region.x + region.width);
            max_y = max_y.max(region.y + region.height);
        }
        let mut root = QuadNode::new_branch(0, 0, max_x, max_y);
        for region in &self.regions {
            let mut leaf = QuadNode::new_leaf(
                region.x,
                region.y,
                region.width,
                region.height,
                region.color,
                region.pattern_hash
            );
            let _ = leaf.split(self.grid_cell_size);
            let _ = root.add_child(leaf);
        }
        self.quadtree = Some(root);
    }

    /// Finally flatten regions into a SixelImage with a 2D pixel buffer.
    pub fn create_image(&mut self) -> Result<SixelImage, SixelError> {
        if !self.got_dcs {
            return Err(SixelError::MissingDcs);
        }
        let mut max_x = 0;
        let mut max_y = 0;
        for region in &self.regions {
            max_x = max_x.max(region.x + region.width);
            max_y = max_y.max(region.y + region.height);
        }
        let mut pixels = vec![vec![Pixel{on:false,color:0}; max_x]; max_y];
        for region in &self.regions {
            if let Some(hash) = region.pattern_hash {
                let pattern_byte = hash as u8;
                for row in 0..region.height.min(SIXEL_HEIGHT) {
                    if self.pattern_dict.is_bit_set(pattern_byte, row) {
                        for col in 0..region.width {
                            let py = region.y + row;
                            let px = region.x + col;
                            pixels[py][px] = Pixel{on:true, color: region.color};
                        }
                    }
                }
                // If region.height > 6, we do not repeat bits. We just fill the first 6 lines.
            } else if region.is_solid {
                for y in region.y..(region.y + region.height) {
                    for x in region.x..(region.x + region.width) {
                        pixels[y][x] = Pixel{on:true, color:region.color};
                    }
                }
            }
        }
        let img = SixelImage {
            pixels,
            color_registers: mem::take(&mut self.color_registers),
        };
        Ok(img)
    }
}

/// A convenience function for decoding a slice of SixelEvents with the region-based approach.
pub fn decode_sixel_events(events: &[SixelEvent], max_width: Option<usize>, max_height: Option<usize>) -> Result<SixelImage, SixelError> {
    let mut decoder = SixelDecoder::new();
    if let Some(w) = max_width {
        decoder.set_max_width(w);
    }
    if let Some(h) = max_height {
        decoder.set_max_height(h);
    }
    for ev in events {
        decoder.handle_event(ev.clone())?;
    }
    decoder.create_image()
}

impl Default for SixelDecoder {
    fn default() -> Self {
        Self::new()
    }
}
