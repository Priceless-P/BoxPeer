import React from 'react';
import { Table, Button, Space } from 'antd';

interface FileData {
  key: string;
  title: string;
  status: string;
  peers: number;
}

const data: FileData[] = [
  {
    key: '1',
    title: 'Example Video',
    status: 'Active',
    peers: 10,
  },
  // Add more file data
];

const FileManagement: React.FC = () => {
  const columns = [
    { title: 'Title', dataIndex: 'title', key: 'title' },
    { title: 'Status', dataIndex: 'status', key: 'status' },
    { title: 'Active Peers', dataIndex: 'peers', key: 'peers' },
    {
      title: 'Action',
      key: 'action',
      render: () => (
        <Space size="middle">
          <Button>Edit</Button>
          <Button danger>Delete</Button>
        </Space>
      ),
    },
  ];

  return <Table style={{marginTop: '50px'}}columns={columns} dataSource={data} />;
};

export default FileManagement;
