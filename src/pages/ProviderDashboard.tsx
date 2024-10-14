import { useEffect, useState } from 'react';
import { Layout, Button, Statistic, Row, Col, message, Table, Input, Typography, Radio, Spin, Menu } from 'antd';
import { CloudUploadOutlined, ClusterOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/tauri';
import { readBinaryFile } from '@tauri-apps/api/fs';
import { open } from '@tauri-apps/api/dialog';
import { useNavigate } from "react-router-dom";
import { useKeylessAccounts } from "../core/useKeylessAccounts.ts";
import GoogleLogo from "../components/GoogleLogo";
import { collapseAddress } from "../core/utils";
import { ProvidedFile, Peer } from './types.ts';
import { useFileManager } from '../utils/fileUtils.tsx';
import { getReward, getTotalEarned, upload_content } from '../core/contracts.ts';
import { Aptos, AptosConfig, Network, U64 } from "@aptos-labs/ts-sdk";

const aptosConfig = new AptosConfig({ network: Network.TESTNET });
const aptos = new Aptos(aptosConfig);
const { Header, Content, Sider } = Layout;

const ProviderDashboard: React.FC = () => {
    const [peerId, setPeerId] = useState<string>('');
    const [peers, setPeers] = useState<Peer[]>([]);
    const [selectedFile, setSelectedFile] = useState<ProvidedFile | null>(null);
    const [accountBalance, setAccountBalance] = useState<string>('');
    const [fileTitle, setFileTitle] = useState<string>('');
    const [fileDescription, setFileDescription] = useState<string>('');
    const [providedFiles, setProvidedFiles] = useState<ProvidedFile[]>([]);
    const [consumerFeeInput, setConsumerFeeInput] = useState<number>(0);
    const [isPaid, setIsPaid] = useState(false);
    const [totalAmountSpent, setTotalAmountSpent] = useState<number>(0);
    const [loading, setLoading] = useState<boolean>(false);
    const [sharedFiles, setSharedFiles] = useState<any[]>([]);
    const [otherFiles, setOtherFiles] = useState<any[]>([]);
    const [totalEarned, setTotalEarned] = useState<number>(0);
    const navigate = useNavigate();

    const { fileObjects, copyToClipboard, fetchFileMetadata } = useFileManager();
    const { activeAccount, disconnectKeylessAccount } = useKeylessAccounts();

    useEffect(() => {
        const metadata = async () => {
            if (fileObjects.length === 0) {
                await fetchFileMetadata();
            }
        };
        metadata();
    }, [fileObjects, fetchFileMetadata]);

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
            startListening()
        }
    }, [activeAccount, navigate]);

    const startListening = async () => {
        try {
            const id = await invoke<string>('start_listening');
            setPeerId(id);
        } catch (error: any) {
            message.error(error);
        }
    };

    useEffect(() => {
        const fetchPeers = async () => {
            try {
                const peersList = await invoke<Peer[]>('list_peers');
                setPeers(peersList);
                // message.success('Fetched connected peers');
            } catch (error) {
                // message.error('Failed to fetch peers');
            }
        };
        fetchPeers();

        const intervalId = setInterval(fetchPeers, 30000);
        return () => clearInterval(intervalId);
    }, []);

    const invokeFiles = async () => {
        setLoading(true);
        try {
            const cidStrings = fileObjects.map((file) => file.cid);
            await classifyFilesByBlockPresence(cidStrings);
        } catch (error) {
            console.error("Error invoking files:", error);
        } finally {
            setLoading(false);
        }
    };

    const classifyFilesByBlockPresence = async (cidStrings: string[]) => {
        const shared = [];
        const others = [];
        for (const cid of cidStrings) {
            const exists = await invoke<boolean>('has_file', { cid });
            const file = fileObjects.find(f => f.cid === cid);
            if (exists) {
                shared.push(file);
            } else {
                others.push(file);
            }
        }
        setSharedFiles(shared);
        setOtherFiles(others);
    };

    const lockFile = async (cid: string) => {
        setLoading(true);
        try {
            const result = await invoke<string>('lock_file', { cid });
            message.info(result);
            const fileToMove = otherFiles.find(file => file.cid === cid);
            const amount = new U64(fileToMove.feePaid * 0.1);
            if (activeAccount) {
                const reward = await getReward(activeAccount, cid, amount);
                message.success(reward)
            }


            if (fileToMove) {
                setSharedFiles([...sharedFiles, fileToMove]);
                setOtherFiles(otherFiles.filter(file => file.cid !== cid));
            }
        } catch (error) {
            console.error("Error locking file:", error);
            message.error("Failed to lock file");
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        invokeFiles();
    }, [fileObjects]);

    const columns = [
        {
            title: 'CID',
            dataIndex: 'cid',
            key: 'cid',
        },
        {
            title: 'Action',
            key: 'action',
            render: (_: any, record: any) => (
                <Button onClick={() => lockFile(record.cid)}>Lock File</Button>
            ),
        },
    ];

    const sharedColumns = [
        // {
        //     title: 'Name',
        //     dataIndex: 'title',
        //     key: 'name',
        // },
        {
            title: 'CID',
            dataIndex: 'cid',
            key: 'cid',
        },
        {
            title: 'Cost',
            dataIndex: 'consumerFee',
            key: 'consumerFee',
            render: (text: string) => {
                const feeInAPT = parseInt(text) === 0 ? 'Free' : (parseInt(text) / 1e8).toFixed(1) + ' APT';
                return <p>{feeInAPT}</p>;
            },
        },
    ];
    useEffect(() => {
        const totalSpent = sharedFiles.reduce((acc, file) => acc + (parseInt(file.feePaid) || 0), 0);
        setTotalAmountSpent(totalSpent / 1e8);

        const amountEarned = async (address: string) => {
            const totalEarned = await getTotalEarned(address);
            setTotalEarned(parseFloat(totalEarned))
        }
        if (activeAccount) {
            amountEarned(activeAccount.accountAddress.toString());
        }

    }, [sharedFiles]);

    const userDataString = localStorage.getItem('@aptos-connect/keyless-accounts');
    let userData = null;

    if (userDataString) {
        userData = JSON.parse(userDataString);
    } else {
        console.warn('No user data found in local storage.');
    }
    const username = userData?.state?.accounts?.[0]?.idToken?.decoded?.given_name || 'Unknown User';

    // Provide a file to the network
    const provideFile = async () => {
        if (!selectedFile) {
            message.error('Please select a file first');
            return;
        }

        try {
            // Read the binary content of the selected file
            setLoading(true);
            const owner_name = username;
            const fileData = await readBinaryFile(selectedFile.name);
            const fileType = selectedFile.type;
            const fileSizeMB = selectedFile.size / (1024 * 1024);
            const APTOS_DECIMALS = 100_000_000;
            let cid: string = await invoke('upload_file', { filePath: selectedFile.name, file_data: fileData });
            const feePaid: U64 = new U64(Math.ceil(fileSizeMB / 100) * APTOS_DECIMALS);

            const consumerFee: U64 = new U64(consumerFeeInput * APTOS_DECIMALS);
            if (!activeAccount) {
                message.error('No active account found. Please log in.');
                return;
            }
            let result = await upload_content(activeAccount, cid.toString(), feePaid, consumerFee, fileType.toString(), owner_name, fileDescription, fileTitle);
            message.success(result);
            setProvidedFiles([...providedFiles, selectedFile]);
        } catch (error: any) {
            console.error(error);
            message.error(`Failed to provide file: ${error}`);
        }
        finally {
            setLoading(false)
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
            }
        } catch (error: any) {
            message.error(error);
        }
    };

    const columnStyle = {
        padding: '16px',
        borderRadius: '8px',
        boxShadow: '0 4px 8px #2E073F',
        transition: 'transform 0.2s',
        color: '#fff',
    };

    return (
        <Layout style={{ minHeight: '100vh' }}>
            <Header style={{ color: '#fff', fontSize: '1.5rem', background: '#2E073F', height: '55px', display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '0 16px' }}>
                <Typography.Title level={3} style={{ margin: 0, color: '#fff' }}>Dashboard</Typography.Title>
                <div style={{ display: 'flex', alignItems: 'center' }}>
                    <div style={{ padding: '16px', textAlign: 'center' }}>
                        <GoogleLogo />
                    </div>
                    <Typography.Text
                        onClick={() => {
                            if (activeAccount?.accountAddress) {
                                copyToClipboard(activeAccount.accountAddress.toString());
                            } else {
                                message.error('No active account address found.');
                            }
                        }}
                        style={{ cursor: 'pointer', color: '#fff', marginRight: '16px' }}
                    >
                        {activeAccount ? collapseAddress(activeAccount.accountAddress.toString()) : 'Not logged in'}
                    </Typography.Text>
                    <Button onClick={disconnectKeylessAccount} style={{ marginLeft: '8px' }}>Logout</Button>
                </div>
            </Header>

            <Layout>
                <Sider width={150} style={{ background: '#2E073F' }}>

                    <Menu style={{ background: '#2E073F' }} >
                        <Menu.Item key="1" style={{ background: '#2E073F' }}>
                            <span style={{ color: 'white', cursor: 'pointer' }} onClick={() => copyToClipboard('https://www.aptosfaucet.com')}>
                                Get Test APT
                            </span>
                        </Menu.Item>
                        <Menu.Item key="2" style={{ background: '#2E073F' }}>
                            <span style={{ color: 'white', cursor: 'pointer' }} onClick={() => copyToClipboard('https://explorer.aptoslabs.com/?network=testnet')}>
                                Explorer
                            </span>
                        </Menu.Item>
                    </Menu>
                </Sider>

                <Layout style={{ padding: '20px 24px' }}>
                    <Content>
                        {loading ? <Spin /> : (
                            <>
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
                                                </div>
                                            </Col>
                                            <Col span={8}>
                                                <div style={columnStyle}>
                                                    <Statistic title="Wallet Balance" value={accountBalance} />
                                                </div>
                                            </Col>
                                            <Col span={8}>
                                                <div style={columnStyle}>
                                                    <Statistic title="Total Files Shared" value={sharedFiles.length} />
                                                </div>
                                            </Col>
                                        </Row>

                                        <Row gutter={16} style={{ marginTop: 16 }}>
                                            <Col span={8}>
                                                <div style={columnStyle}>
                                                    <Statistic title="Total Amount Spent" value={`${totalAmountSpent.toFixed(2)} APT`} />
                                                </div>
                                            </Col>
                                            <Col span={8}>
                                                <div style={columnStyle}>
                                                    <Statistic title="Total Amount Earned" value={`${totalEarned} APT`} />
                                                </div>
                                            </Col>
                                            <Col span={8}>
                                                <div style={columnStyle}>
                                                    <Statistic title="Connected Peers" value={peers.length} />

                                                </div>
                                            </Col>
                                        </Row>
                                    </Col>
                                </Row>

                                <Row gutter={16} style={{ marginTop: 40 }}>
                                    <Col span={24}>
                                        <Typography.Title level={4}>Upload a File</Typography.Title>

                                        {/* File Selection */}
                                        <Button
                                            icon={<CloudUploadOutlined />}
                                            onClick={handleFileChange}
                                            style={{ marginBottom: 16 }}
                                        >
                                            Select File
                                        </Button>
                                        {fileTitle && (
                                            <Col>
                                                <Typography.Text type="secondary">Selected File: {selectedFile?.name}</Typography.Text>
                                            </Col>
                                        )}

                                        <Input
                                            placeholder="Enter file name"
                                            value={fileTitle}
                                            onChange={(e) => setFileTitle(e.target.value)}
                                            style={{ marginTop: 16, width: '100%' }}
                                            maxLength={100}
                                        />
                                        <Typography.Text type="secondary" style={{ fontSize: 12 }}>
                                            Maximum 100 characters
                                        </Typography.Text>

                                        {/* File Description */}
                                        <Input.TextArea
                                            placeholder="Enter a short description"
                                            value={fileDescription}
                                            onChange={(e) => setFileDescription(e.target.value)}
                                            style={{ marginTop: 16, width: '100%' }}
                                            autoSize={{ minRows: 3, maxRows: 6 }}
                                        />

                                        {/* Pricing Option */}
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
                                            <Radio value="free">Free</Radio>
                                            <Radio value="paid">Paid</Radio>
                                        </Radio.Group>

                                        {/* Consumer Fee Input for Paid Option */}
                                        {isPaid && (
                                            <Input
                                                type="number"
                                                placeholder="Enter fee in APT"
                                                value={consumerFeeInput}
                                                onChange={(e) => setConsumerFeeInput(parseFloat(e.target.value))}
                                                style={{ marginTop: 16, width: '100%' }}
                                                min={0}
                                                step={0.01}
                                            />
                                        )}
                                        {isPaid && (
                                            <Typography.Text type="secondary" style={{ fontSize: 12 }}>
                                                Specify the fee in APT (e.g., 0.1). Leave blank for free content.
                                            </Typography.Text>
                                        )}

                                        {/* Upload Button */}
                                        <Button
                                            type="primary"
                                            onClick={provideFile}
                                            disabled={!fileTitle || (isPaid && (!consumerFeeInput || consumerFeeInput <= 0))}
                                            style={{ marginTop: 16 }}
                                        >
                                            Upload
                                        </Button>
                                    </Col>
                                </Row>


                                <Row gutter={16} style={{ marginTop: 30, width: '100%' }}>
                                    <Col span={24} style={{ width: '100%' }}>
                                        <Typography.Title level={4}>Shared Files</Typography.Title>
                                        <Table dataSource={sharedFiles} columns={sharedColumns} pagination={false} style={{ width: '100%' }} />
                                    </Col>
                                </Row>

                                <Row gutter={16} style={{ marginTop: 30 }}>
                                    <Col span={24}>
                                        <Typography.Title level={4}>Other Files</Typography.Title>
                                        <Table dataSource={otherFiles} columns={columns} pagination={false} style={{ width: '100%' }} />
                                    </Col>
                                </Row>
                            </>
                        )}
                    </Content>
                </Layout>
            </Layout>
        </Layout>
    );
};

export default ProviderDashboard;
