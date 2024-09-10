# penrose_bbarker_contrib

Various penrose tools that I use (or have used) in my penrose-based
window manager.


## Misc

Utilities that don't have a better place to be are currently in lib.rs.


## Logging

See log.rs and [this blog post](https://bbarker.unison-services.cloud/s/bbblog/posts/concise-error-absolution-in-rust);
it is opinionated in the sense that it currently has no way to configure
it to log anywhere other than `$HOME/.penrose.log`, but this could likely
be changed if there is interest.
