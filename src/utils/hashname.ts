// async function generateKeyAndStore() {
//     const key = await crypto.subtle.generateKey(
//         {
//             name: "AES-GCM",
//             length: 256,
//         },
//         true, // Extractable
//         ["encrypt", "decrypt"] // Usages
//     );

//     // Export the key to a raw format
//     const exportedKey = await crypto.subtle.exportKey("raw", key);
//     const keyArray = new Uint8Array(exportedKey);

//     const base64Key = btoa(String.fromCharCode.apply(null, keyArray));

//     console.log("Key",base64Key);
// }

// generateKeyAndStore();

// async function importKey(base64Key: string) {
//     const binaryKey = Uint8Array.from(atob(base64Key), c => c.charCodeAt(0));
//     return await crypto.subtle.importKey(
//         "raw",
//         binaryKey,
//         {
//             name: "AES-GCM",
//         },
//         true,
//         ["encrypt", "decrypt"]
//     );
// }

// async function encryptName(name, key) {
//     const encoder = new TextEncoder();
//     const data = encoder.encode(name);
//     const iv = crypto.getRandomValues(new Uint8Array(12)); // Initialization vector

//     const ciphertext = await crypto.subtle.encrypt(
//         {
//             name: "AES-GCM",
//             iv: iv,
//         },
//         key,
//         data
//     );

//     return { iv: Array.from(iv), ciphertext: Array.from(new Uint8Array(ciphertext)) };
// }

// async function decryptName(encryptedData, key) {
//     const { iv, ciphertext } = encryptedData;
//     const decodedIv = new Uint8Array(iv);
//     const decodedCiphertext = new Uint8Array(ciphertext);

//     const decryptedData = await crypto.subtle.decrypt(
//         {
//             name: "AES-GCM",
//             iv: decodedIv,
//         },
//         key,
//         decodedCiphertext
//     );

//     const decoder = new TextDecoder();
//     return decoder.decode(decryptedData);
// }


export async function hashName(name: string) {
    const encoder = new TextEncoder();
    const data = encoder.encode(name);
    const hashBuffer = await crypto.subtle.digest('SHA-256', data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = hashArray.map(byte => byte.toString(16).padStart(2, '0')).join('');
    return hashHex;
}
