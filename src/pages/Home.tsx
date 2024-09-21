import { Layout, Typography, Button, Row, Col, Card, Statistic } from 'antd';
import { DatabaseOutlined, CloudServerOutlined, UserOutlined } from '@ant-design/icons';
import Nav from './Nav';
import './HomePage.css';


const { Content } = Layout;
const { Title, Text } = Typography;

const Home = () => {
    return (
        <Layout style={{ minHeight: '100vh' }}>
            <Nav />
            {/* Hero Section */}
            <Content style={{  padding: '50px 0' }}>
                <Row justify="center" align="middle" gutter={32}>
                    <Col xs={24} md={12} style={{ textAlign: 'center' }} >
                        <Title level={1}>Decentralized Content Delivery Network</Title>
                        <Text style={{ fontSize: '18px', color: '#555' }}>
                            Empowering fast and secure content delivery through a decentralized peer-to-peer network.
                        </Text>
                        <Row justify="center" gutter={16} style={{ marginTop: '30px' }}>
                            <Col>
                                <Button type="primary" size="large">Get Started</Button>
                            </Col>
                            <Col>
                                <Button type="default" size="large">Learn More</Button>
                            </Col>
                        </Row>
                    </Col>
                    <Col xs={16} md={8} style={{ textAlign: 'center' }}>
                        <img src="/assets/cdn.png" alt="CDN" style={{ maxWidth: '100%', height: 'auto' }} />
                    </Col>
                </Row>
            </Content>

            {/* Quick Stats Section */}
            <Content style={{ padding: '50px 50px' }}>
                <Row gutter={16}>
                    <Col span={8}>
                        <Card hoverable>
                            <Statistic
                                title="Data Distributed"
                                value={500}
                                precision={0}
                                prefix={<DatabaseOutlined />}
                                suffix="TB"
                            />
                        </Card>
                    </Col>
                    <Col span={8}>
                        <Card hoverable>
                            <Statistic
                                title="Active Nodes"
                                value={1200}
                                precision={0}
                                prefix={<CloudServerOutlined />}
                            />
                        </Card>
                    </Col>
                    <Col span={8}>
                        <Card hoverable>
                            <Statistic
                                title="Satisfied Users"
                                value={300000}
                                precision={0}

                                prefix={<UserOutlined />}
                            />
                        </Card>
                    </Col>
                </Row>
            </Content>

            {/* How It Works Section */}
            <Content style={{ padding: '50px 50px' }}>
                <Title level={2} style={{ textAlign: 'center', marginBottom: '40px' }}>
                    How It Works
                </Title>
                <Row gutter={16}>
                    <Col span={8}>
                        <Card title="Step 1: Upload Content" bordered={false} hoverable>
                            <Text>
                                Content providers upload their files to the network. Files are broken into smaller chunks and distributed across multiple peers.
                            </Text>
                        </Card>
                    </Col>
                    <Col span={8}>
                        <Card title="Step 2: Distribute" bordered={false} hoverable>
                            <Text>
                                The network automatically distributes the chunks to peers, ensuring redundancy and availability.
                            </Text>
                        </Card>
                    </Col>
                    <Col span={8}>
                        <Card title="Step 3: Deliver" bordered={false} hoverable>
                            <Text>
                                When users request content, chunks are retrieved from multiple peers, assembled, and delivered quickly and securely.
                            </Text>
                        </Card>
                    </Col>
                </Row>
            </Content>


        </Layout>
    );
};

export default Home;
