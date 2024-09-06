import React from 'react';
import { Row, Col, Card, Statistic, Typography } from 'antd';
import { GlobalOutlined, ThunderboltOutlined, FireOutlined, UsergroupAddOutlined } from '@ant-design/icons';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

const data = [
  { name: 'Jan', downloads: 400 },
  { name: 'Feb', downloads: 300 },
  { name: 'Mar', downloads: 200 },
  { name: 'Apr', downloads: 278 },
  { name: 'May', downloads: 189 },
  { name: 'Jun', downloads: 239 },
];
const { Title } = Typography;

interface AnalyticsProps {
  activePeers: number;
  peerGeoDistribution: string;
  downloadSpeed: number; // in Mbps
  uploadSpeed: number;   // in Mbps
  contentPopularity: number; // Number of times content accessed/downloaded
}

const Analytics: React.FC<AnalyticsProps> = ({
  activePeers,
  peerGeoDistribution,
  downloadSpeed,
  uploadSpeed,
  contentPopularity
}) => {
  return (
    <div style={{ padding: '50px 50px', backgroundColor: '#f0f2f5' }}>
      <Title level={2} style={{ textAlign: 'center', marginBottom: '40px', color: '#001529' }}>
        Analytics
      </Title>
      <Row gutter={16}>
        {/* Active Peers */}
        <Col span={12}>
          <Card hoverable>
            <Statistic
              title="Active Peers"
              value={activePeers}
              precision={0}
              valueStyle={{ color: '#3f8600' }}
              prefix={<UsergroupAddOutlined />}
            />
          </Card>
        </Col>

        {/* Peer Geographic Distribution */}
        <Col span={12}>
          <Card hoverable>
            <Statistic
              title="Peer Geographic Distribution"
              value={peerGeoDistribution}
              valueStyle={{ color: '#3f8600' }}
              prefix={<GlobalOutlined />}
            />
          </Card>
        </Col>
      </Row>

      <Row gutter={16} style={{ marginTop: '30px' }}>
        {/* Download/Upload Speed */}
        <Col span={12}>
          <Card hoverable>
            <Statistic
              title="Download Speed"
              value={downloadSpeed}
              precision={2}
              suffix="Mbps"
              valueStyle={{ color: '#3f8600' }}
              prefix={<ThunderboltOutlined />}
            />
            <Statistic
              title="Upload Speed"
              value={uploadSpeed}
              precision={2}
              suffix="Mbps"
              valueStyle={{ color: '#3f8600' }}
              prefix={<ThunderboltOutlined />}
              style={{ marginTop: '20px' }}
            />
          </Card>
        </Col>

        {/* Content Popularity */}
        <Col span={12}>
          <Card hoverable>
            <Statistic
              title="Content Popularity"
              value={contentPopularity}
              precision={0}
              valueStyle={{ color: '#3f8600' }}
              prefix={<FireOutlined />}
            />
          </Card>
        </Col>
        <Col span={24}>
        <Card title="Download Frequency">
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={data}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="name" />
              <YAxis />
              <Tooltip />
              <Legend />
              <Line type="monotone" dataKey="downloads" stroke="#8884d8" />
            </LineChart>
          </ResponsiveContainer>
        </Card>
      </Col>
      </Row>
    </div>
  );
};

export default Analytics;
