import init, { decodeTiff } from "../pkg/obscura_image.js";

async function process(file) {
  const outputDiv = document.getElementById("output");
  outputDiv.innerHTML = "Processing...";

  try {
    // Read the file as an ArrayBuffer
    const arrayBuffer = await file.arrayBuffer();
    const tiffData = new Uint8Array(arrayBuffer);

    console.log(
      `Input TIFF file: ${file.name}, size: ${tiffData.length} bytes`,
    );

    // Decode TIFF to PNG
    const result = decodeTiff(tiffData);

    console.log("Decoding result:", result);

    let html = `
                        <div class="success">
                            Successfully processed TIFF!<br>
                            Total images found: ${result.total_images}<br>
                            Successfully decoded: ${result.images.length}<br>
                            ${result.errors.length > 0 ? `Failed to decode: ${result.errors.length}` : ""}
                        </div>
                    `;

    // Show errors if any
    if (result.errors.length > 0) {
      html += "<h3>Decode Errors:</h3><ul>";
      for (const error of result.errors) {
        html += `<li><strong>Image ${error.image_index}:</strong> ${error.error_message}</li>`;
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

        html += `
                                <div style="margin: 20px 0; border: 1px solid #ddd; padding: 15px; border-radius: 5px;">
                                    <h4>Image ${image.metadata.image_index}</h4>
                                    <ul>
                                        <li><strong>Dimensions:</strong> ${image.metadata.width} × ${image.metadata.height}</li>
                                        <li><strong>Color Type:</strong> ${image.metadata.color_type}</li>
                                        <li><strong>Bit Depth:</strong> ${image.metadata.bit_depth}-bit</li>
                                        <li><strong>PNG Size:</strong> ${image.png_data.length.toLocaleString()} bytes</li>
                                    </ul>
                                    <img src="${url}" alt="Decoded PNG ${i}" style="max-width: 100%; border: 1px solid #ddd;" />
                                    <br>
                                    <a href="${url}" download="${file.name.replace(/\.(tiff?|TIF+)$/i, "")}_${image.metadata.image_index}.png">Download PNG ${image.metadata.image_index}</a>
                                </div>
                            `;
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
  document.getElementById("tiffInput").addEventListener("change", async (e) => {
    const file = e.target.files[0];
    if (!file) return;
    await process(file);
  });
}

run().catch(console.error);
