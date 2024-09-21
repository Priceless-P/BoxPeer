import React, { useState } from 'react';
import { Button, message, Image, Modal } from 'antd';
import { DownloadOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/tauri';
import {Link} from "react-router-dom";
import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useKeylessAccounts } from "../core/useKeylessAccounts";
import GoogleLogo from "../components/GoogleLogo";
import { collapseAddress } from "../core/utils";

const PeerDashboard: React.FC = () => {
    const [contentHash, setContentHash] = useState<string>('');
    const [fileUrl, setFileUrl] = useState<string>('');
    const [fileType, setFileType] = useState<string>('image/png');  // Default to PNG
    const [isModalVisible, setIsModalVisible] = useState(false);

    const navigate = useNavigate();

    const { activeAccount, disconnectKeylessAccount } = useKeylessAccounts();

    useEffect(() => {
        if (!activeAccount) navigate("/");
    }, [activeAccount, navigate]);

    // Request file from the network
    const requestFile = async () => {
        if (!contentHash) {
            message.error('Please provide a valid content hash.');
            return;
        }

        try {
            // Invoke the backend command to fetch the file
            const fileData = await invoke<Uint8Array>('get_file', { contentHash: contentHash });
            console.log("Received file data:", fileData); // Debugging: log the received data

            // Create a Blob from the binary data
            const blob = new Blob([new Uint8Array(fileData)], { type: fileType });
            console.log("Created blob:", blob); // Debugging: log the created blob

            // Create a URL for the Blob
            const url = URL.createObjectURL(blob);
            setFileUrl(url);  // URL for displaying file

            message.success('File retrieved successfully!');
            setIsModalVisible(true);
        } catch (error: any) {
            message.error('Failed to fetch the file.');
            console.error("Error fetching file:", error); // Debugging: log the error
        }
    };

    // Download the file
    const downloadFile = () => {
        if (!fileUrl) return;

        // Create a hidden anchor tag to download the file
        const link = document.createElement('a');
        link.href = fileUrl;
        link.download = `file_from_network.${fileType.split('/')[1]}`; // Get extension from fileType
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link); // Remove the link after download
    };

    const handleFileTypeChange = (type: string) => {
        setFileType(type);
    };

    return (
        <div style={{padding: '20px'}}>
            <div className="grid gap-2">
                {activeAccount ? (
                    <div
                        className="flex justify-center items-center border rounded-lg px-8 py-2 shadow-sm cursor-not-allowed">
                        <GoogleLogo/>
                        {collapseAddress(activeAccount?.accountAddress.toString())}
                    </div>
                ) : (
                    <p>Not logged in</p>
                )}
                <button
                    className="flex justify-center bg-red-50 items-center border border-red-200 rounded-lg px-8 py-2 shadow-sm shadow-red-300 hover:bg-red-100 active:scale-95 transition-all"
                    onClick={disconnectKeylessAccount}
                >
                    Logout
                </button>
            </div>
            <div>
            </div>
                <h2>Request File</h2>

                {/* Input field for content hash */}
                <input
                    style={{width: '60%', marginRight: '10px'}}
                    placeholder="Enter content hash"
                    value={contentHash}
                    onChange={(e) => setContentHash(e.target.value)}
                />

                {/* Dropdown for selecting file type */}
                <select onChange={(e) => handleFileTypeChange(e.target.value)} defaultValue="image/png">
                    <option value="image/png">PNG</option>
                    <option value="image/jpeg">JPEG</option>
                    <option value="audio/mpeg">MP3</option>
                    <option value="video/mp4">MP4</option>
                    <option value="application/pdf">PDF</option>
                </select>

                {/* Button to request the file */}
                <Button
                    type="primary"
                    icon={<DownloadOutlined/>}
                    onClick={requestFile}
                    style={{marginLeft: '10px'}}
                >
                    Fetch File
                </Button>

                {/* Modal to display file preview */}
                <Modal
                    title="File Preview"
                    visible={isModalVisible}
                    footer={[
                        <Button key="download" type="primary" onClick={downloadFile}>
                            Download File
                        </Button>,
                    ]}
                    onCancel={() => setIsModalVisible(false)}
                >
                    {/* Display different file types based on MIME */}
                    {fileType.startsWith('image/') && <Image src={fileUrl} alt="Requested file"/>}
                    {fileType.startsWith('audio/') && (
                        <audio controls src={fileUrl}>
                            Your browser does not support the audio tag.
                        </audio>
                    )}
                    {fileType.startsWith('video/') && (
                        <video controls width="100%" src={fileUrl}>
                            Your browser does not support the video tag.
                        </video>
                    )}
                    {fileType === 'application/pdf' && (
                        <iframe src={fileUrl} title="PDF File" width="100%" height="500px"></iframe>
                    )}
                </Modal>
                <Link to="/dashboard">Other Dashbaord</Link>
            </div>
            );
            };

            export default PeerDashboard;
