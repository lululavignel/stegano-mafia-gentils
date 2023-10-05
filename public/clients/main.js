import init, { get_image_width, get_image_height, decrypt_message } from '../../pkg/stegano_project.js';

async function run() {
    // Initialize the WebAssembly module asynchronously
    console.log('Before init');
    await init();
    console.log('After init');
    
    const imageElement = document.getElementById('myImage');
    const base64Image = await imageElementToBase64(imageElement);
    
    const width = get_image_width(base64Image);
    const height = get_image_height(base64Image);
    console.log('Image Width:', width);
    console.log('Image Height:', height);

    const secretMessage = decrypt_message(base64Image);
    console.log('Secret Message:', secretMessage);
    console.log("prout");
}

// Helper function to convert an HTMLImageElement to base64
async function imageElementToBase64(imageElement) {
    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d');
    canvas.width = imageElement.width;
    canvas.height = imageElement.height;
    context.drawImage(imageElement, 0, 0, canvas.width, canvas.height);

    // Convert the canvas content to a base64-encoded string
    return canvas.toDataURL('image/png').split(',')[1];
}

run();
