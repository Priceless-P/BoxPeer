# BoxPeer

BoxPeer is a content distribution network (CDN) and decentralized file-sharing platform. It uses a peer-to-peer network to distribute content across multiple nodes, ensuring data redundancy, availability, and security. Users can upload files, serve as nodes to share others' content, and earn rewards for participating in the network. The platform integrates with Aptos blockchain technology for a good user experience.

## Features

- **Decentralized Storage:** Files uploaded to BoxPeer are stored across multiple nodes, providing redundancy and fault tolerance.
- **Content Distribution Network (CDN):** Ensures fast and efficient file delivery by caching files across participating nodes.
- **File Streaming:** Supports text, audio, and video streaming directly from the network.
- **Content Categorization:** Allows content to be classified as paid or free, with restricted access to premium content until payment is made.
- **File Persistence:** Locally caches files for offline access and automatically uploads them to the network upon reconnection.
- **Node Rewards:** Users can earn rewards by serving as nodes and sharing content.
- **Modern Web Interface:** Offers a nice, user-friendly interface for browsing, uploading, and managing files.

## Installation

### Download the Application

You can download pre-built versions of BoxPeer for your operating system:

- Windows (64-bit): Coming Soon
- macOS (64-bit): Coming Soon
- Linux (64-bit): Coming Soon

> **Note:** The built versions is not currently available due to ongoing issues. Running the development version is recommended for a now.

### Run in Development Mode
1. **Clone the repo**
    ```bash
       git clone https://github.com/Priceless-P/BoxPeer.git
       cd ./BoxPeer
    ```
2. **Set env**
    ```bash
     cp .env-sample .env
    ```
2. **Install the dependencies:**

   ```bash
   npm install
   ```

2. **Start the Tauri app:**

   ```bash
   npm run tauri dev
   ```

   or

   ```bash
   cargo tauri dev
   ```

This will launch the development environment for BoxPeer, allowing you to use the application with live reload.

## Usage

### Uploading Files

1. Navigate to the BoxPeer web interface.
2. Click on the "Upload" button and select the files you want to upload.
3. Choose whether the content is paid or free.
4. Once uploaded, the file will be distributed across the network.

### Sharing Files and Earning Rewards

Nodes on the BoxPeer network can earn rewards by locking content in their dashboard and sharing it with others. Here’s how it works:

1. Go to your dashboard and you will see a list contents available in the network, lock the file you want to share. This process makes your node a distributor for that content.
2. You will a certain amount of APT fo locking files. The more content you lock, the more rewards you earn.

### Viewing Content

1. Visit [boxpeer](https://boxpeer.net/contents) Browse the available content from the main page.
2. Click on a file to view its details.
3. For paid content, connect your wallet to make a payment before accessing the file.



## Acknowledgments

- **[libp2p](https://github.com/libp2p/rust-libp2p):** For enabling peer-to-peer networking.


### For more in-depth technical details
- See [this](./SPECS.md)
