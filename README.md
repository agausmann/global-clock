# Global Clock

A universal 24-hour analog clock that tells you the time everywhere!

[Run in Browser](https://agausmann.github.io/global-clock/)

This app visualizes the rotation and axial tilt of the Earth relative to the
Sun in real time (assuming your system's clock is accurate).

The Earth is rendered with an [azimuthal projection] centered on the South
pole, so it will rotate clockwise and so lines of longitude will radiate in
straight lines from the center. It is roughly aligned to a 24 hour clock, with
midnight at the top and noon at the bottom.

There is also a clock face which displays the local time (again, depending on
your system clock) in a 24-hour analog format, so the hour hand is synchronized
with the rotation of the Earth.

## Build it yourself

### Dependencies

- The [Rust toolchain](https://www.rust-lang.org/tools/install)

### Build instructions

Download the repository and run this command in the root directory: 

```sh
cargo build --release
```

The executable can be found at `target/release/global-clock`.

## Credits

Earth textures are obtained from the [Solar Textures] pack, by Solar System
Scope. It is made available under the terms of the [Attribution 4.0
International][CC BY 4.0] license.

Inspired by <https://xkcd.com/now>:

![XKCD: Now](https://imgs.xkcd.com/comics/now.png)

[azimuthal projection]: https://en.wikipedia.org/wiki/Map_projection#Azimuthal_.28projections_onto_a_plane.29
[Solar Textures]: https://www.solarsystemscope.com/textures/
[CC BY 4.0]: https://creativecommons.org/licenses/by/4.0/
