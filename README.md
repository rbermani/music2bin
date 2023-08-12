# music2bin

[![Rust](https://github.com/rbermani/music2bin/actions/workflows/rust.yml/badge.svg)](https://github.com/rbermani/music2bin/actions/workflows/rust.yml)

An application for condensing musical content represented using MusicXML into a minimal condensed binary format targeted at ML transformer model training.

The objective is to fit entire musical compositions (jazz, pop, classical), represented on a grand staff, into the token limit of a transformer.

One of the limitations to existing generative musical language model approaches is a result of the token limit imposed by transformer based models. It's not currently computationally feasible to fit an entire musical composition in an uncompressed audio form into the token limit of a transformer in the way that is possible with a 512x512 bitmap.

Reducing the informational content down to a condensed format that contains the core "spirit" of the composition from beginning, middle to end, and training a model on this more cohesive, holistic form is more likely to produce impressive results.

After a model is generated through training, the application can be used to transform the generated binary output back into a MusicXML representation that will be viewable by tools such as MuseScore, and easily converted back into a MIDI representation for playback.

