# Interclip server

This repository contains the code for the rewrite of Interclip's server. The current server implementation at https://github.com/interclip/interclip is a garbled mess of PHP and JavaScript, and is hard to maintain. This rewrite is written in Rust, and in addition to the meme of "Rewrite it in Rust", it also has a lot of benefits:

- It's fast
- My IDE can actually understand the code
- There's more validation and type safety