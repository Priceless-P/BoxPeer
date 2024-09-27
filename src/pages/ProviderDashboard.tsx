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
import { upload_content } from '../core/contracts.ts';
import { Aptos, AptosConfig, Network, U64 } from "@aptos-labs/ts-sdk";

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
            const id = await invoke<string>('start_listening');
            setPeerId(id);
            message.success(`Started listening with Peer ID: ${id}`);
        } catch (error: any) {
            message.error(error);
        }
    };

    // Fetch connected peers
    const fetchPeers = async () => {
        try {
            const peersList = await invoke<Peer[]>('list_peers');
            setPeers(peersList);
            message.success('Fetched connected peers');
        } catch (error) {
            message.error('Failed to fetch peers');
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
            const contentHash = filename;
            const fileSizeMB = selectedFile.size / (1024 * 1024);
            let p = await invoke('upload_file', { filePath: selectedFile.name, file_data: fileData });
            console.log(p);
            const feePaid: U64 = new U64(Math.ceil(fileSizeMB / 100));

            let nodes = ["0x9e99af6d494ca087085ae7b14c0f422b41b53e62db5b68708bbb2286f8abcb45"];
            const consumerFee: U64 = new U64(consumerFeeInput);
            if (!activeAccount) {
                message.error('No active account found. Please log in.');
                return;
            }
            let result = await upload_content(activeAccount, contentHash, nodes, feePaid, consumerFee);
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
        // backgroundColor: '#fff',
        padding: '16px', // Padding around the content
        borderRadius: '8px', // Rounded corners
        boxShadow: '0 4px 8px #2E073F', // Box shadow
        transition: 'transform 0.2s',
        color: '#fff',// Smooth transform effect
    };

    return (
        <Layout style={{ minHeight: '100vh' }}>
            <Header style={{ color: '#fff', fontSize: '1.5rem', background:'#2E073F', height: '50px'}}>
                {/* Provider Dashboard */}
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

                        <Button onClick={disconnectKeylessAccount} style={{ marginTop:'30px' }}>Logout</Button>
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
                            <Statistic title="Connected Peers" value={peers.length} />
                            <Button type="primary" onClick={fetchPeers} style={{ marginTop: 16 }}>
                                Get Peers
                            </Button>
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
                            <Statistic title="Wallet Balance" value={accountBalance} />
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
                    </Content>
                </Layout>
            </Layout>
        </Layout>
    );
};

export default ProviderDashboard;
