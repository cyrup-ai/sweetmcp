
This is the authoritative, minimal checklist of all work required to achieve a fully working, production-quality browser agent, as per project conventions and protocol.  
**No TODOs, "come back later", or non-production comments are allowed in the codebase.**

---

## Browser State, Screenshot, and Clickable Items Extraction

- [ ] In `src/agent/custom_agent.rs`, extract bounding boxes and labels for all visually distinct elements from the screenshot using the output of the segmentation model.
- [ ] In `src/agent/custom_agent.rs`, for each detected region, use OCR (e.g., Kalosm OCR or another OCR crate) to extract visible text from the screenshot region.
- [ ] In `src/agent/custom_agent.rs`, build a structured list of clickable/interactable elements, including their bounding box, label/type, and text, as a Rust struct or JSON object.
- [ ] In `src/agent/custom_agent.rs`, pass this structured list to the LLM in the agent prompt as the "Interactive elements" section, replacing or supplementing the current DOM-based list.
- [ ] In `src/agent/custom_agent.rs`, ensure the LLM prompt and agent state include both the base64 screenshot and the structured list of visual elements.
- [ ] In `src/controller/custom_controller.rs`, update the controller to map between visual elements (from segmentation) and DOM selectors, so actions can be executed on the correct elements. This must maintain a mapping between bounding boxes/labels and DOM `data-mcp-index` attributes.

---

**No item may be marked complete until it is fully implemented, tested, and production-ready.**
