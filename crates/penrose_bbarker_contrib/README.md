# penrose_bbarker_contrib

Various penrose tools that I use (or have used) in my penrose-based
window manager. While the preference is to publish generally useful
extensions and other code in the main [penrose](https://github.com/sminez/penrose/)
repository, there may be various reasons not to do so, or the code here may
incubate a while before I attempt to move it over.

## Misc

Utilities that don't have a better place to be are currently in lib.rs.


## Logging

See log.rs and [this blog post](https://bbarker.unison-services.cloud/s/bbblog/posts/concise-error-absolution-in-rust);
it is opinionated in the sense that it currently has no way to configure
it to log anywhere other than `$HOME/.penrose.log`, but this could likely
be changed if there is interest.


## Workspaces

See workspaces.rs for utilities relating to workspaces, such as retrieving
workspace apps.

## Menus

Several [dmenu-rs](https://github.com/Shizcow/dmenu-rs)-based menus I've
written for navigation and common tasks, including a finder to locate a
workspace by process name or window title.
