import React from 'react';
import { Layout, Menu, Card, Row, Col, Statistic, Typography, Button, Table } from 'antd';
import { DashboardOutlined, DollarOutlined, CloudOutlined, RadarChartOutlined } from '@ant-design/icons';
import { PeerDashboardProps } from './types';
import NetworkGraph from './NetworkGraph';

const { Header, Sider, Content } = Layout;
const { Title } = Typography;

const PeerDashboard: React.FC<PeerDashboardProps> = () => {
  // Sample data for the table
  const transactionData = [
    { key: '1', date: '2024-08-23', amount: '0.02 APT', content: 'File A' },
    { key: '2', date: '2024-08-22', amount: '0.05 APT', content: 'File B' },
    { key: '3', date: '2024-08-21', amount: '0.01 APT', content: 'File C' },
  ];

  const transactionColumns = [
    { title: 'Date', dataIndex: 'date', key: 'date' },
    { title: 'Amount', dataIndex: 'amount', key: 'amount' },
    { title: 'Content', dataIndex: 'content', key: 'content' },
  ];

  const nodes = [
    { id: 'Node 1', group: 1 },
    { id: 'Node 2', group: 2 },
    { id: 'Node 3', group: 1 },
    { id: 'Node 4', group: 2 },
  ];

  const links = [
    { source: 'Node 1', target: 'Node 2' },
    { source: 'Node 1', target: 'Node 3' },
    { source: 'Node 3', target: 'Node 4' },
  ];

  return (
    <Layout style={{ minHeight: '100vh', }}>
      {/* Sidebar */}
      <Sider collapsible style={{ backgroundColor: '#fff' }}>
        <Menu style={{ backgroundColor: '#fff' }} defaultSelectedKeys={['nodeStatus']} mode="inline">
          <Menu.Item key="nodeStatus" icon={<DashboardOutlined />}>
            Node Status
          </Menu.Item>
          <Menu.Item key="earnings" icon={<DollarOutlined />}>
            Earnings
          </Menu.Item>
          <Menu.Item key="cacheManagement" icon={<CloudOutlined />}>
            Cache Management
          </Menu.Item>
          <Menu.Item key="networkMonitoring" icon={<RadarChartOutlined />}>
            Network Monitoring
          </Menu.Item>
        </Menu>
      </Sider>

      <Layout>
        {/* Top Navigation Bar */}
        <Header style={{ backgroundColor: '#fff', padding: 0, textAlign: 'center' }}>
          <Title level={3} style={{ margin: '16px 0' }}>Peer/Node Dashboard</Title>
        </Header>

        {/* Main Content */}
        <Content style={{ margin: '16px' }}>
          <Row gutter={16}>
            {/* Node Status */}
            <Col span={6}>
              <Card hoverable>
                <Statistic title="Node Status" value="Online" valueStyle={{ color: '#3f8600' }} />
                <Statistic title="Total Content Served" value={250} suffix="items" style={{ marginTop: '16px' }} />
                <Statistic title="Current Workload" value={34} suffix="chunks" style={{ marginTop: '16px' }} />
              </Card>
            </Col>

            {/* Earnings */}
            <Col span={6}>
              <Card hoverable>
                <Statistic title="Total Earnings" value="0.08 APT" valueStyle={{ color: '#1890ff' }} />
                <Table dataSource={transactionData} columns={transactionColumns} pagination={false} style={{ marginTop: '16px' }} />
              </Card>
            </Col>

            {/* Cache Management */}
            <Col span={6}>
              <Card hoverable>
                <Title level={5}>Cache Management</Title>
                <Button type="primary" style={{ marginBottom: '16px' }}>Prioritize Content</Button>
                <Button danger>Clear Cache</Button>
              </Card>
            </Col>

            {/* Network Monitoring */}
            <Col span={8}>
            <Card hoverable>
                <Title level={5}>Network Monitoring</Title>
                <NetworkGraph />
              </Card>
            </Col>
          </Row>
        </Content>
      </Layout>
    </Layout>
  );
};

export default PeerDashboard;
