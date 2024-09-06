import React from 'react';
import { Layout, Input, Row, Col, Card, Button, Typography } from 'antd';
import { useNavigate } from 'react-router-dom';

import Nav from './Nav';
import Footer_ from './Footer';

const { Content } = Layout;
const { Text } = Typography;
const { Search } = Input;

interface ContentItem {
  id: number;
  title: string;
  type: string;
  size: string;
  cost: string;
  description: string;
}

const sampleContent: ContentItem[] = [
  { id: 1, title: "Video Tutorial on Aptos", type: "Video", size: "500MB", cost: "5 APT", description: "Learn the basics of Aptos blockchain." },
  { id: 2, title: "Aptos Whitepaper", type: "Document", size: "2MB", cost: "Free", description: "Detailed documentation on Aptos blockchain." },
];

const Contents: React.FC = () => {
  const navigate = useNavigate();

  const handleSearch = (value: string) => {
    console.log('Search value:', value);
  };

  const handleContentSelect = (content: ContentItem) => {
    navigate(`/content/${content.id}`, { state: { content } });
  };

  return (
    <Layout style={{ minHeight: '100vh', backgroundColor: '#fff' }}>
      <Nav />

      <Content style={{ backgroundColor: '#fff', padding: '20px 50px' }}>
        <Search placeholder="Search for content..." enterButton onSearch={handleSearch} />
      </Content>

      <Content style={{ padding: '20px 50px' }}>
        <Row gutter={[16, 16]}>
          {sampleContent.map((item) => (
            <Col span={8} key={item.id}>
              <Card
                hoverable
                title={item.title}
                actions={[
                  <Button type="primary" onClick={() => handleContentSelect(item)}>View Details</Button>,
                ]}
              >
                <Text>Type: {item.type}</Text><br />
                <Text>Size: {item.size}</Text><br />
                <Text>Cost: {item.cost}</Text>
              </Card>
            </Col>
          ))}
        </Row>
      </Content>

      <Footer_ />
    </Layout>
  );
};

export default Contents;
