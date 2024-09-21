import {useEffect, useState} from 'react';
import { Layout, Button, Card, Statistic, Row, Col, message, Table, Upload } from 'antd';
import { CloudUploadOutlined, ClusterOutlined, FileOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/tauri';
import { readBinaryFile } from '@tauri-apps/api/fs';
import { open } from '@tauri-apps/api/dialog';
import {Link, useNavigate} from "react-router-dom";
import {useKeylessAccounts} from "../core/useKeylessAccounts.ts";
import GoogleLogo from "../components/GoogleLogo";
import { collapseAddress } from "../core/utils";


const { Header, Content } = Layout;

interface Peer {
  id: string;
}

interface ProvidedFile {
  name: string;
  size: number;
  type: string;
}

const ProviderDashboard: React.FC = () => {
  const [listeningAddress, setListeningAddress] = useState<string>('');  // Listening address (from backend)
  const [peerId, setPeerId] = useState<string>('');  // Peer ID
  const [peers, setPeers] = useState<Peer[]>([]);  // List of connected peers
  const [selectedFile, setSelectedFile] = useState<ProvidedFile | null>(null);
  const [filename, setFilename] = useState<string>('');
  const [providedFiles, setProvidedFiles] = useState<ProvidedFile[]>([]);  // List of provided files

  const navigate = useNavigate();

  const { activeAccount, disconnectKeylessAccount } = useKeylessAccounts();

  useEffect(() => {
    if (!activeAccount) navigate("/");

  }, [activeAccount, navigate]);

  // Start listening and get Peer ID
  const startListening = async () => {
    try {
      const id = await invoke<string>('start_listening');
      setPeerId(id);
      setListeningAddress(id);
      console.log("Public key", activeAccount?.publicKey.idCommitment);
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
      const fileData = await readBinaryFile(selectedFile.name);
      await invoke('provide_file', { path: selectedFile.name, fileName: filename, contentHash: filename, file_data: fileData });
      message.success('File provided successfully');
      setProvidedFiles([...providedFiles, selectedFile]);  // Add file to provided files list
    } catch (error: any) {
      message.error(error);
    }
  };

  // Handle file selection
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

  return (
      <Layout style={{ minHeight: '100vh' }}>
        <Header style={{ color: '#fff', fontSize: '1.5rem' }}>
          Provider Dashboard
        </Header>

        <Content style={{ padding: '20px 50px' }}>
          <Row gutter={16}>
            {/* Listening Info */}
            <Col span={6}>
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
            </Col>
            <Col span={8}>
              <Card>
                <Statistic title="Listening Address" value={listeningAddress || 'N/A'} prefix={<ClusterOutlined/>}/>
                <p><Statistic title="Peer ID" value={peerId || 'N/A'}/></p>
                <Button type="primary" onClick={startListening} style={{marginTop: 16}}>
                  Start Listening
                </Button>
              </Card>
            </Col>

            {/* Connected Peers */}
            <Col span={8}>
              <Card>
                <Statistic title="Connected Peers" value={peers.length}/>
                <Button type="primary" onClick={fetchPeers} style={{marginTop: 16}}>
                  Get Peers
                </Button>
                <Table
                    dataSource={peers}
                    columns={[{ title: 'Peer ID', dataIndex: 'id', key: 'id' }]}
                    pagination={false}
                    style={{ marginTop: 16 }}
                />
              </Card>
            </Col>

            {/* File Uploader */}
            <Col span={8}>
              <Card title="Provide a File">
                <Upload
                    beforeUpload={() => false}
                    onChange={handleFileChange}
                    maxCount={1}
                    accept=".jpg,.png,.txt,.pdf,.mp3,.mp4,.mkv,*"
                >
                  <Button icon={<CloudUploadOutlined />}>Select File</Button>
                </Upload>
                {filename && <p>Selected File: {filename}</p>}
                <Button
                    type="primary"
                    onClick={provideFile}
                    disabled={!selectedFile}
                    style={{ marginTop: 16 }}
                    icon={<FileOutlined />}
                >
                  Provide File
                </Button>
              </Card>
            </Col>
          </Row>

          {/* Provided Files */}
          <Row gutter={16} style={{ marginTop: 30 }}>
            <Col span={24}>
              <Card title="Provided Files">
                <Table
                    dataSource={providedFiles}
                    columns={columns}
                    rowKey="name"
                    pagination={false}
                />
              </Card>
            </Col>
          </Row>
        </Content>
        <Link to="/node-dashboard">Other Dashbaord</Link>
      </Layout>
  );
};

export default ProviderDashboard;
