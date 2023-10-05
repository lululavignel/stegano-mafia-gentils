import init, { oui } from '../../pkg/stegano_project.js';

async function run() {
    // Initialize the WebAssembly module asynchronously
    console.log('Before init');
    await init();
    console.log('After init');
    

    // Now you can safely call the 'oui' function
    const result = oui(2, 5);
    console.log(result);
}

run();
