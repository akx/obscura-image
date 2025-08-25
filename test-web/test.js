import init, { decodeTiff } from "../pkg/obscura_image.js";

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

async function process(file) {
  const outputDiv = document.getElementById("output");
  outputDiv.innerHTML = "Processing...";

  try {
    // Read the file as an ArrayBuffer
    const arrayBuffer = await file.arrayBuffer();
    const fileData = new Uint8Array(arrayBuffer);
    console.log(`Input file: ${file.name}, size: ${fileData.length} bytes`);
    const result = decodeTiff(fileData);
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
<img src="${url}" alt="Decoded PNG ${i}" /><br>
<a href="${url}" download="${file.name}_${info.image_index}.png">Download PNG ${info.image_index}</a>
</div>`;
      }
    }

    outputDiv.innerHTML = html;
  } catch (error) {
    console.error("Error:", error);
    outputDiv.innerHTML = `<div class="error">Error: ${error.message || error}</div>`;
  }
}

async function run() {
  await init();
  document.getElementById("fileInput").addEventListener("change", async (e) => {
    const file = e.target.files[0];
    if (!file) return;
    await process(file);
  });
}

run().catch(console.error);
