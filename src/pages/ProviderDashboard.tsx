import { useEffect, useState } from 'react';
import { Layout, Button, Statistic, Row, Col, message, Table, Input, Typography, Radio } from 'antd';
import { CloudUploadOutlined, ClusterOutlined, FileOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/tauri';
import { readBinaryFile } from '@tauri-apps/api/fs';
import { open } from '@tauri-apps/api/dialog';
import { Link, useNavigate } from "react-router-dom";
import { useKeylessAccounts } from "../core/useKeylessAccounts.ts";
import GoogleLogo from "../components/GoogleLogo";
import { collapseAddress } from "../core/utils";
import { upload_content, queryRegistry, fetchAllContentCIDs } from '../core/contracts.ts';
import { Aptos, AptosConfig, Network, U64, AccountAddress } from "@aptos-labs/ts-sdk";

const aptosConfig = new AptosConfig({ network: Network.TESTNET });
const aptos = new Aptos(aptosConfig);
const { Header, Content, Sider } = Layout;
const { Title } = Typography;

interface Peer {
    id: string;
}

interface ProvidedFile {
    name: string;
    size: number;
    type: string;
}

const ProviderDashboard: React.FC = () => {
    const [peerId, setPeerId] = useState<string>('');
    const [peers, setPeers] = useState<Peer[]>([]);
    const [selectedFile, setSelectedFile] = useState<ProvidedFile | null>(null);
    const [accountBalance, setAccountBalance] = useState<string>('');
    const [filename, setFilename] = useState<string>('');
    const [providedFiles, setProvidedFiles] = useState<ProvidedFile[]>([]);
    const [consumerFeeInput, setConsumerFeeInput] = useState<number>(0);
    const [isPaid, setIsPaid] = useState(false);
    const [fileCid, setFileCid] = useState<string>('');
    const [previewContent, setPreviewContent] = useState<JSX.Element | null>(null); // New state for preview content
    const navigate = useNavigate();

    const { activeAccount, disconnectKeylessAccount } = useKeylessAccounts();

    useEffect(() => {
        if (!activeAccount) {
            navigate("/");
        } else {
            const getBalance = async () => {
                try {
                    const balance = await aptos.getAccountAPTAmount({
                        accountAddress: activeAccount.accountAddress
                    });
                    const formattedBalance = (balance / 1e8).toFixed(4);
                    setAccountBalance(`${formattedBalance} APT`);
                } catch (error) {
                    console.error("Error fetching account balance:", error);
                }
            };
            getBalance();
        }
    }, [activeAccount, navigate]);

    // Start listening and get Peer ID
    const startListening = async () => {
        try {
        //const walletAddress =
            const id = await invoke<string>('start_listening', );
            setPeerId(id);
            message.success(`Listening with Peer ID: ${id}`);
        } catch (error: any) {
            message.error(error);
        }
    };

    useEffect(() => {
        const fetchPeers = async () => {
            try {
                const peersList = await invoke<Peer[]>('list_peers');
                setPeers(peersList);
                message.success('Fetched connected peers');
            } catch (error) {
                message.error('Failed to fetch peers');
            }
        };
        fetchPeers();

        const intervalId = setInterval(fetchPeers, 30000);
        return () => clearInterval(intervalId);
    }, []);

    // New function to handle file preview based on type
    const previewFile = async (cid: string) => {
        try {
        const contents = await fetchAllContentCIDs()
        console.log("Contents: ", contents)
            const fileData: number[]  = await invoke<number[]>('request_file', { cid, savePath: '/Admin' });

            const byteArray = new Uint8Array(fileData);
            const accountAddress = activeAccount?.accountAddress?.toString() ?? 'defaultAccountAddress'; // Replace with your default value
            const fileType = await queryRegistry(accountAddress, cid);

            console.log("file type", fileType);
            if (fileType) {
            // Convert the file data to a Blob
            const blob = new Blob([byteArray], { type: getMimeType(fileType) });
            const fileURL = URL.createObjectURL(blob);

                       // Preview the file based on its MIME type
switch (fileType) {
    case 'jpeg':
    case 'jpg':
    case 'png':
        setPreviewContent(<img src={fileURL} alt="preview" style={{ maxWidth: '100%' }} />);
        break;
    // case 'text/plain':
    case 'pdf':
        setPreviewContent(<iframe src={fileURL} style={{ width: '100%', height: '500px' }} />);
        break;
    case 'mpeg':
        setPreviewContent(<audio controls src={fileURL} />);
        break;
    // case 'video/mp4':
    case 'x-matroska':
        setPreviewContent(<video controls style={{ maxWidth: '100%' }} src={fileURL} />);
        break;
    default:
        setPreviewContent(<a href={fileURL} download>Download the file</a>);
}
            } else {
            message.error("File type is empty")}
            return



        } catch (error: any) {
            console.error('Failed to preview file:', error);
            message.error('Failed to preview file', error);
        }
    };

    // Function to map file extension to MIME type
    const getMimeType = (fileType: string | undefined) => {
        switch (fileType) {
            case 'jpg': return 'image/jpeg';
            case 'png': return 'image/png';
            case 'txt': return 'text/plain';
            case 'pdf': return 'application/pdf';
            case 'mp3': return 'audio/mpeg';
            case 'mp4': return 'video/mp4';
            case 'mkv': return 'video/x-matroska';
            default: return 'application/octet-stream';
        }
    };

    // Provide a file to the network
    const provideFile = async () => {
        if (!selectedFile) {
            message.error('Please select a file first');
            return;
        }

        try {
            // Read the binary content of the selected file
            const fileData = await readBinaryFile(selectedFile.name);
            const fileType = selectedFile.type;
            const fileSizeMB = selectedFile.size / (1024 * 1024);
            const APTOS_DECIMALS = 100_000_000;
            let cid: string = await invoke('upload_file', { filePath: selectedFile.name, file_data: fileData });
            console.log("CID", cid);
            const feePaid: U64 = new U64(Math.ceil(fileSizeMB / 100) * APTOS_DECIMALS);

            let nodes = ["0xcc9ee328719b87be92c5012eff8b2eb3a0e3f82024af6a97658cb431ba707bda"];
            function toMoveAddress(address: string): string {
                if (address.length !== 66 || !address.startsWith("0x")) {
                    throw new Error("Invalid address format. Must be 32-byte hex with 0x prefix.");
                }
                return address;
            }
            let moveAddresses = nodes.map(toMoveAddress);
            const consumerFee: U64 = new U64(consumerFeeInput * APTOS_DECIMALS);
            if (!activeAccount) {
                message.error('No active account found. Please log in.');
                return;
            }
            let result = await upload_content(activeAccount, cid.toString(), moveAddresses, feePaid, consumerFee, fileType.toString());
            message.success(result);
            setProvidedFiles([...providedFiles, selectedFile]);
        } catch (error: any) {
            console.error(error);
            message.error(`Failed to provide file: ${error}`);
        }
    };

    const handleFileChange = async () => {
        try {
            const selectedFilePath = await open({
                multiple: false,
                filters: [{ name: 'All Files', extensions: ['jpg', 'png', 'txt', 'pdf', 'mp3', 'mp4', 'mkv', '*'] }]
            });

            if (typeof selectedFilePath === 'string') {
                const fileSize = await readBinaryFile(selectedFilePath).then(file => file.byteLength);
                const newFile = { name: selectedFilePath, size: fileSize, type: selectedFilePath.split('.').pop() || 'unknown' };
                setSelectedFile(newFile);
                setFilename(selectedFilePath.split('/').pop() || '');
            }
        } catch (error: any) {
            message.error(error);
        }
    };

    const copyToClipboard = (text: string) => {
        navigator.clipboard.writeText(text).then(() => {
            message.success("Copied to clipboard!");
        }).catch(err => {
            message.error("Failed to copy: " + err);
        });
    };

    const columns = [
        {
            title: 'File Name',
            dataIndex: 'name',
            key: 'name',
        },
        {
            title: 'File Size',
            dataIndex: 'size',
            key: 'size',
            render: (size: number) => `${(size / 1024).toFixed(2)} KB`,
        },
        {
            title: 'File Type',
            dataIndex: 'type',
            key: 'type',
        },
    ];

    const columnStyle = {
        padding: '16px',
        borderRadius: '8px',
        boxShadow: '0 4px 8px #2E073F',
        transition: 'transform 0.2s',
        color: '#fff',
    };

    return (
        <Layout style={{ minHeight: '100vh' }}>
            <Header style={{ color: '#fff', fontSize: '1.5rem', background: '#2E073F', height: '50px' }}>
            </Header>

            <Layout>
                <Sider width={110} style={{ background: '#2E073F', }}>
                    <div style={{ padding: '16px', textAlign: 'center' }}>
                        <GoogleLogo />
                        <Typography.Text
                            onClick={() => {
                                if (activeAccount?.accountAddress) {
                                    copyToClipboard(activeAccount.accountAddress.toString());
                                } else {
                                    message.error('No active account address found.');
                                }
                            }}
                            style={{ cursor: 'pointer', color: '#fff', }}
                        >
                            {activeAccount ? collapseAddress(activeAccount?.accountAddress.toString()) : 'Not logged in'}
                        </Typography.Text>

                        <Button onClick={disconnectKeylessAccount} style={{ marginTop: '30px' }}>Logout</Button>
                    </div>
                </Sider>

                <Layout style={{ padding: '20px 24px' }}>
                    <Content>
                    <Row gutter={16}>
            <Col span={24}>
                <Row gutter={16}>
                    <Col span={8}>
                        <div onClick={() => peerId && copyToClipboard(peerId)} style={{ cursor: 'pointer', ...columnStyle }}>
                            <Statistic
                                title="Peer ID"
                                value={peerId ? `${peerId.slice(0, 6)}...` : 'N/A'}
                                prefix={<ClusterOutlined />}
                            />

                        <Button type="primary" onClick={startListening} style={{ marginTop: 10 }}>
                            Start Listening
                        </Button>
                        </div>
                    </Col>
                    <Col span={8}>
                        <div style={columnStyle}>
                            <Statistic title="Wallet Balance" value={accountBalance} />
                        </div>
                    </Col>
                    <Col span={8}>
                        <div style={columnStyle}>
                            <Statistic title="Total Files Provided" value={providedFiles.length} />
                        </div>
                    </Col>
                </Row>

                <Row gutter={16} style={{ marginTop: 16 }}>
                    <Col span={8}>
                        <div style={columnStyle}>
                            <Statistic title="Total Amount Spent" value={`0 APT`} />
                        </div>
                    </Col>
                    <Col span={8}>
                        <div style={columnStyle}>
                            <Statistic title="Total Amount Earned" value={`0 APT`} />
                        </div>
                    </Col>
                    <Col span={8}>
                        <div style={columnStyle}>
                            <Statistic title="Connected Peers" value={peers.length} />
                            {/* <Button type="primary" onClick={fetchPeers} style={{ marginTop: 16 }}>
                                Get Peers
                            </Button> */}
                        </div>
                    </Col>
                </Row>
            </Col>
        </Row>

                        <Row
            gutter={16}
            style={{
                marginTop: 40,
                justifyContent: 'flex-end' // Aligns the content to the right
            }}>
            <Col span={24}>
                <Typography.Title level={4}>Upload a File</Typography.Title>
                <Button icon={<CloudUploadOutlined />} onClick={handleFileChange}>
                    Select File
                </Button>
                <Col>
                    {filename && <Typography.Text>Selected File: {filename}</Typography.Text>}
                </Col>

                {/* Radio buttons to select if the content is paid or free */}
                <Radio.Group
                    onChange={(e) => {
                        setIsPaid(e.target.value === 'paid');
                        if (e.target.value === 'free') {
                            setConsumerFeeInput(0);
                        }
                    }}
                    value={isPaid ? 'paid' : 'free'}
                    style={{ marginTop: 16 }}
                >
                    <Radio value="paid">Paid</Radio>
                    <Radio value="free">Free</Radio>
                </Radio.Group>

                {/* Conditional input for consumer fee */}
                {isPaid && (
                    <Input
                        type="number"
                        placeholder="Set consumer fee (APT)"
                        onChange={(e) => setConsumerFeeInput(parseInt(e.target.value))}
                        style={{ marginTop: 16, width: '300px' }}
                    />
                )}

                <Button
                    type="primary"
                    onClick={provideFile}
                    disabled={!filename || (isPaid && !consumerFeeInput)}
                    style={{ marginTop: 16 }}
                >
                    Upload
                </Button>
            </Col>
        </Row>

                        <Row gutter={16} style={{ marginTop: 30 }}>
                            <Col span={24}>
                            <div style={{ textAlign: 'center', marginTop: 20 }}> {/* Centering the title */}
            <Typography.Title level={3}>Uploaded Files in the Network</Typography.Title>
        </div>
                                <Table dataSource={providedFiles} columns={columns} pagination={false} />
                            </Col>
                        </Row>
                        <Row gutter={16} style={{ marginTop: 30 }}>
    {/* <Col span={24}>
        <Typography.Title level={4}>Request a File</Typography.Title>
        <Input
            placeholder="Enter File CID"
            onChange={(e) => setFileCid(e.target.value)} // Use state to track the CID input
            style={{ marginBottom: 16 }}
        />
        <Button
            type="primary"
            onClick={() => requestFile(fileCid)}
            disabled={!fileCid}
        >
            Get File
        </Button>
    </Col> */}
</Row>
                        <Row gutter={16}>
                            {/* Request file preview */}
                            <Col span={24}>
                                <Typography.Title level={4}>Request a File</Typography.Title>
                                <Input
                                    placeholder="Enter File CID"
                                    onChange={(e) => setFileCid(e.target.value)}
                                    style={{ marginBottom: 16 }}
                                />
                                <Button
                                    type="primary"
                                    onClick={() => previewFile(fileCid)}
                                    disabled={!fileCid}
                                >
                                    Preview File
                                </Button>
                            </Col>
                        </Row>

                        {/* File preview section */}
                        <Row gutter={16} style={{ marginTop: 30 }}>
                            <Col span={24}>
                                {previewContent ? (
                                    previewContent
                                ) : (
                                    <Typography.Text>No file preview available</Typography.Text>
                                )}
                            </Col>
                        </Row>
                    </Content>
                </Layout>
            </Layout>
        </Layout>
    );
};

export default ProviderDashboard;
