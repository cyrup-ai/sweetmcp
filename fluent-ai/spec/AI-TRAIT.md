# Specification

## Overview

These are examples of fluent builder interfaces I adore:

```rust
let reply_stream = CompletionProvider::openai()
    .model("o4-mini")
    .system_prompt("You are…") // -> AgentBuilder<MissingCtx>
    .context(doc_index.top_n())// -> AgentBuilder<Ready>
    .tool::<Calc>()            // const-generic tool registration
    .temperature(1.0)
    .completion() // builds CompletionProvider
    .on_chunk( | result | { // executed on each Stream chunk to unwrap
        Ok => result.into_chunk(),
        Err(e) => result.into_err!("agent failed: {e}")
    })
    .chat(
        "Hello! How's the new framework coming?"
    ); // returns unwrapped chunks in CompletionStream processed by on_chunk closure
```


```rust
let csv_records = AsyncTask::emits::<CsvRecord>()
    .with(schema_config)        // Pass dependencies using with()
    .with(processing_stats)
    .with_timeout(60.seconds())           // Configuration next
        collector.of_file("data.csv")
            .with_delimiter(Delimiter::NewLine)
            .into_chunks(100.rows());
        processing_stats.increment_files();
    })
    .receiver(|event, collector| {        // Receiver gets event + collector
        let record = event.data();
        if record.is_valid() {
            collector.collect(record.id, record);
        }
    })
    .await_final_event(|event, collector| {
        OK(result) => collector.collected(),
        ERR(e) => Err(e)
    });
```

```rust
let mut audio_stream = MyTtsEngine::conversation()
    .with_speaker(
        Speaker::speaker("Alice")
            .voice_id(VoiceId::new("voice-uuid"))
            .with_speed_modifier(VocalSpeedMod(0.9))
            .speak("Hello, world!")
            .build()
    )
    .with_speaker(
        Speaker::speaker("Bob")
            .with_speed_modifier(VocalSpeedMod(1.1))
            .speak("Hi Alice! How are you today?"),
        Speaker::speaker("Alice")
            .with_noise_reduction(Denoise::level(0.5))
            .speak("I'm doing great, thanks for asking!")
    )
    .synthesize(|conversation| {
        Ok  => conversation.into_stream(),  // Returns audio stream
        Err(e) => Err(e),
    })
    .await?;  // Single await point

// Process audio samples
while let Some(sample) = audio_stream.next().await {
    // Play sample or save to file
    println!("Audio sample: {}", sample);
}
```

## Finding commonalities

Use sequential thinking to find commonalities
in these builders. Reflect on those. ULTRATHINK.

Now use sequential thinking to create spec/BUILDER.model
and write down your observations on style and structure.

## Domain model

in ./domain/* you'll find type definitions for the ai stack.
Use sequential thinking to review and become familiar with the domain model.

## ARCHITECTURE.md

Create a proposal for my review of the builder architecture over
this domain model.

## Pure Traits + Builders

In this crate you'll create comprehensive traits over this domain model.
Then you'll design traits for the builders.

## Create Builder Traits

Yes! Builders will have traits too.

Set one domain model up completely and show it to me for approval
After that you can blaze through all of them.
