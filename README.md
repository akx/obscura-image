# obscura-image

[![npm](https://img.shields.io/npm/v/obscura-image)](https://www.npmjs.com/package/obscura-image)

Decode esoteric image formats in the browser using WebAssembly.

## Usage

```javascript
import init, { decodeTiff } from 'obscura-image';

await init();

// Decode TIFF to PNG
const result = decodeTiff(tiffData);

// Access decoded images
for (const image of result.images) {
  console.log(image.metadata); // width, height, color_type, bit_depth
  const blob = new Blob([image.png_data], { type: 'image/png' });
  // Use the blob as needed
}
```

## License

MIT