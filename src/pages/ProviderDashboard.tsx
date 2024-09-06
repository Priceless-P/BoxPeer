import React from 'react';
import { Button, Input, Form } from 'antd';
import { invoke } from '@tauri-apps/api/tauri';

interface ProviderDashboardProps {
  startNetwork: () => Promise<void>;
}

const ProviderDashboard: React.FC<ProviderDashboardProps> = ({ startNetwork }) => {
  const handleProvideFile = async (values: { fileName: string; filePath: string }) => {
    try {
      await invoke('provide_file', { fileName: values.fileName, filePath: values.filePath });
      console.log('File provided to the network');
    } catch (error) {
      console.error('Failed to provide file:', error);
    }
  };

  return (
    <div>
      <h1>Provider Dashboard</h1>
      <Button onClick={startNetwork}>Connect to Network</Button>
      <Form onFinish={handleProvideFile} layout="vertical">
        <Form.Item name="fileName" label="File Name" rules={[{ required: true }]}>
          <Input />
        </Form.Item>
        <Form.Item name="filePath" label="File Path" rules={[{ required: true }]}>
          <Input />
        </Form.Item>
        <Form.Item>
          <Button type="primary" htmlType="submit">Provide File</Button>
        </Form.Item>
      </Form>
    </div>
  );
};

export default ProviderDashboard;
