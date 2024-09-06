import React, { useState } from 'react';
import { Layout, Button, List, Rate, Form, Input, Typography, Card } from 'antd';
import { ShoppingCartOutlined } from '@ant-design/icons';
import { useLocation } from 'react-router-dom';

import Nav from './Nav';
import Footer_ from './Footer';

const { Content } = Layout;
const { Title, Text } = Typography;

interface ContentItem {
  id: number;
  title: string;
  type: string;
  size: string;
  cost: string;
  description: string;
}

interface Review {
  author: string;
  comment: string;
  date: string;
  rating: number;
}

const ContentDetails: React.FC = () => {
  const { state } = useLocation();
  const [reviews, setReviews] = useState<Review[]>([]);
  const content: ContentItem = state.content;

  const handleReviewSubmit = (values: { author: string; comment: string; rating: number }) => {
    const newReview: Review = { ...values, date: new Date().toISOString() };
    setReviews([...reviews, newReview]);
  };

  return (
    <Layout style={{ minHeight: '100vh', backgroundColor: '#fff' }}>
      <Nav />

      <Content style={{ padding: '20px 50px', backgroundColor: '#fff' }}>
        <Title level={3}>{content.title}</Title>
        <Text>Type: {content.type}</Text><br />
        <Text>Size: {content.size}</Text><br />
        <Text>Cost: {content.cost}</Text><br />
        <Text>Description: {content.description}</Text><br />
        <Button type="primary" icon={<ShoppingCartOutlined />}>Buy Now</Button>
      </Content>

      <Content style={{ padding: '20px 50px', backgroundColor: '#f0f2f5' }}>
        <Title level={4}>Reviews</Title>
        <List
          className="comment-list"
          header={`${reviews.length} reviews`}
          itemLayout="vertical"
          dataSource={reviews}
          renderItem={review => (
            <li>
              <Card>
                <Rate disabled value={review.rating} />
                <Text strong>{review.author}</Text>
                <p>{review.comment}</p>
                <Text type="secondary">{new Date(review.date).toLocaleString()}</Text>
              </Card>
            </li>
          )}
        />
        <Form onFinish={handleReviewSubmit}>
          <Form.Item name="author" rules={[{ required: true, message: 'Please input your name!' }]}>
            <Input placeholder="Your Name" />
          </Form.Item>
          <Form.Item name="comment" rules={[{ required: true, message: 'Please input your review!' }]}>
            <Input.TextArea rows={4} placeholder="Your Review" />
          </Form.Item>
          <Form.Item name="rating" rules={[{ required: true, message: 'Please rate the content!' }]}>
            <Rate />
          </Form.Item>
          <Form.Item>
            <Button type="primary" htmlType="submit">Submit Review</Button>
          </Form.Item>
        </Form>
      </Content>

      <Footer_ />
    </Layout>
  );
};

export default ContentDetails;
