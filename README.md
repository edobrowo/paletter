## Paletter
A super simple command-line palette quantizer.

**Example**: Generating a 16-colour palette of [Ferris](https://rustacean.net/):
<p align="center">
  <img src="doc/demo.png" alt="crabby boi demo" width="800"/>
</p>

## Installation
[Download precompiled binaries here](https://github.com/edobrowo/paletter/releases). Thanks [cargo-dist](https://opensource.axo.dev/cargo-dist/) :)

## Usage
To generate a palette, simply specify the palette size and an image path.
```sh
paletter 256 "/path/to/your/image.png"
```

You can generate a palette for any number of images.
```sh
paletter 1024 "image1.png" "image2.jpg" "image3.webp"
```

Paletter supports decimal RGB and hexadecimal display formats. RGB is the default display mode.
```sh
paletter 256 "image.png" --rgb --hex --uncolored
```

An alpha channel threshold can be specified to prevent transparent values from counting toward the palette. This is useful in quantizing images with transparent backgrounds.
```sh
paletter 16 "image.svg" --alpha-thresh 255
```

The output can be left unsorted or sorted by HSV. The colored output can be disabled.
```sh
paletter 256 "image.jpg" --rgb --sort --uncolored
```

Paletter can use different quantization methods. Currently, `median-cut` and `octree` are supported, with `median-cut` used by default.
```sh
paletter "image.png" --method octree
```

**Note**: Octree quantization is not guaranteed to produce a palette of the expected size without loss of information. Paletter outputs the final result and indicates the actual palette size at the end of the color list. This issue is less likely to occur as the palette size increases.
