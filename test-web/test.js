import init, { decodeMrc, decodeTiff } from "../pkg/obscura_image.js";

function dumpMetadata(metadata) {
  let html = "<table>";
  for (const [key, value] of [...metadata.entries()].sort(([a], [b]) =>
    a.localeCompare(b),
  )) {
    html += `<tr><td>${key}</td><td>${value}</td></tr>`;
  }
  html += "</table>";
  return html;
}

async function wrapProcess(processor) {
  const outputDiv = document.getElementById("output");
  outputDiv.innerHTML = "Processing...";

  try {
    outputDiv.innerHTML = await processor();
  } catch (error) {
    console.error("Error:", error);
    outputDiv.innerHTML = `<div class="error">Error: ${error.message || error}</div>`;
  }
}

async function processImageData(filename, arrayBuffer) {
  const fileData = new Uint8Array(arrayBuffer);
  console.log(`Input file: ${filename}, size: ${fileData.length} bytes`);
  const result = filename.toLowerCase().endsWith(".mrc")
    ? decodeMrc(fileData)
    : decodeTiff(fileData);
  console.log("Decoding result:", result);
  let html = `
<div class="success">
    Successfully processed image!<br>
    Total images found: ${result.total_images}<br>
    Successfully decoded: ${result.images.length}<br>
    ${result.errors.length > 0 ? `Failed to decode: ${result.errors.length}` : ""}
</div>`;

  // Show file metadata if available
  if (result.metadata) {
    html += "<h3>File Metadata:</h3>";
    html += `<div class="metadata">${dumpMetadata(result.metadata)}</div>`;
  }

  // Show errors if any
  if (result.errors.length > 0) {
    html += "<h3>Decode Errors:</h3><ul>";
    for (const error of result.errors) {
      html += `<li><strong>Image ${error.image_index}:</strong> ${error.message}</li>`;
    }
    html += "</ul>";
  }

  // Show successfully decoded images
  if (result.images.length > 0) {
    html += "<h3>Decoded Images:</h3>";

    for (let i = 0; i < result.images.length; i++) {
      const image = result.images[i];
      const blob = new Blob([image.png_data], { type: "image/png" });
      const url = URL.createObjectURL(blob);

      let info = image.info;
      html += `
<div class="image-container">
<h4>Image ${info.image_index}</h4>
<ul>
    <li><strong>Dimensions:</strong> ${info.width} × ${info.height}</li>
    <li><strong>Color Type:</strong> ${info.color_type}</li>
    <li><strong>Bit Depth:</strong> ${info.bit_depth}-bit</li>
    <li><strong>PNG Size:</strong> ${image.png_data.length.toLocaleString()} bytes</li>
</ul>
`;

      // Show image metadata if available
      if (info.metadata) {
        html += "<h5>Image Metadata:</h5>";
        html += `<div class="image-metadata">${dumpMetadata(info.metadata)}</div>`;
      }

      html += `
<img src="${url}" alt="Decoded PNG ${i}" />
<br>
<a href="${url}" download="${filename}_${info.image_index}.png">Download PNG ${info.image_index}</a>
</div>`;
    }
  }
  return html;
}

async function processUrl(url, filename) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch ${url}: ${response.statusText}`);
  }
  const arrayBuffer = await response.arrayBuffer();
  return processImageData(filename, arrayBuffer);
}

async function processUpload(file) {
  const arrayBuffer = await file.arrayBuffer();
  return await processImageData(file.name, arrayBuffer);
}

const exampleFiles = [
  { name: "EMD-3197.mrc", path: "../tests/EMD-3197.mrc" },
  { name: "bilevel.tiff", path: "../tests/bilevel.tiff" },
  {
    name: "broken-at-byte-6155.tiff",
    path: "../tests/broken-at-byte-6155.tiff",
  },
  { name: "cmyk-lzw.tiff", path: "../tests/cmyk-lzw.tiff" },
  { name: "gray8.tiff", path: "../tests/gray8.tiff" },
  { name: "multipage.tiff", path: "../tests/multipage.tiff" },
  { name: "rgb16.tiff", path: "../tests/rgb16.tiff" },
  { name: "rgb8.tiff", path: "../tests/rgb8.tiff" },
];

function setupExampleFiles() {
  const container = document.getElementById("exampleFiles");
  for (const file of exampleFiles) {
    const link = document.createElement("a");
    link.href = "#";
    link.textContent = file.name;
    link.style.display = "inline-block";
    link.style.marginRight = "10px";
    link.style.marginBottom = "5px";
    link.addEventListener("click", (e) => {
      e.preventDefault();
      void wrapProcess(() => processUrl(file.path, file.name));
    });
    container.appendChild(link);
  }
}

async function run() {
  await init();
  setupExampleFiles();
  document.getElementById("fileInput").addEventListener("change", (e) => {
    const file = e.target.files[0];
    if (!file) return;
    void wrapProcess(() => processUpload(file));
  });
}

run().catch(console.error);
