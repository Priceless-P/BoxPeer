import React, { useState } from 'react';
import { Form, Input, Button, Upload, Select, Radio } from 'antd';
import { UploadOutlined } from '@ant-design/icons';

const { TextArea } = Input;
const { Option } = Select;

const UploadContentForm: React.FC = () => {
  const [file, setFile] = useState<File | null>(null);

  const onUpload = () => {
    console.log('Uploading:', file);
  };

  return (
    <Form layout="vertical" onFinish={onUpload}>
      <Form.Item label="Title" name="title" rules={[{ required: true, message: 'Please enter a title!' }]}>
        <Input />
      </Form.Item>
      <Form.Item label="Description" name="description">
        <TextArea rows={4} />
      </Form.Item>
      <Form.Item label="Tags" name="tags">
        <Select mode="tags" placeholder="Add tags">
          <Option value="video">Video</Option>
          <Option value="document">Document</Option>
          <Option value="music">Music</Option>
        </Select>
      </Form.Item>
      <Form.Item label="Permissions" name="permissions">
        <Radio.Group>
          <Radio value="public">Public</Radio>
          <Radio value="private">Private</Radio>
          <Radio value="paid">Paid</Radio>
        </Radio.Group>
      </Form.Item>
      <Form.Item label="Upload File" name="file" rules={[{ required: true, message: 'Please upload a file!' }]}>
        <Upload beforeUpload={(file) => { setFile(file); return false; }}>
          <Button icon={<UploadOutlined />}>Click to Upload</Button>
        </Upload>
      </Form.Item>
      <Form.Item>
        <Button type="primary" htmlType="submit">Upload</Button>
      </Form.Item>
    </Form>
  );
};

export default UploadContentForm;
