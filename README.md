# Logger
---
An opinionated lightweight wrapper of the [log](https://docs.rs/log/latest/log/) crate used for debugging purposes.

### Purpose:

The reason I built this library is to gain a better understanding behind one of the guiding principles in Software Engineering: monitoring an application. From this reusable and lightweight foundation, I can begin scaffolding the Rust-based Compute libraries that are my real aim.

### Features:

1. Log Levels
2. Terminal as Log Target
3. Generic Type Formatting
4. (Coming Soon) Advanced Formatting
    a. Key-Value Pairs and Objects
    b. Tableing
5. (Coming Soon) File as Log Target
    a. Store in local .txt file
    b. Upload to AWS S3

### Usage:

The adoption of this library is quite straightforward, requiring a simple initialization step prior to logging as usual with [log](https://docs.rs/log/latest/log/).

```rust
use log::{debug, error, warn, info};

fn main() {
    logger::init();
    debug!("This is a debug statement!");
    error!("This is a error statement!");
    warn!("This is a warn statement!");
    info!("This is a info statement!");
}
```

[INSERT EXAMPLE OUTPUT HERE]