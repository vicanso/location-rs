import { useState, FC } from "react";
import {
  ConfigProvider,
  theme,
  Card,
  Layout,
  Input,
  message,
  Descriptions,
  Form,
  Select,
  Col,
  Row,
  Checkbox,
  Typography,
  Space,
  Button,
} from "antd";
import { SearchOutlined } from "@ant-design/icons";

import axios, { AxiosError } from "axios";

import "./App.css";

const { defaultAlgorithm, darkAlgorithm } = theme;
const { Header, Content } = Layout;
const { Search } = Input;
const { Paragraph } = Typography;

const isDarkMode = () =>
  window.matchMedia("(prefers-color-scheme: dark)").matches;

const getGithubIcon = (isDarkMode: boolean) => {
  let color = `rgb(0, 0, 0)`;
  if (isDarkMode) {
    color = `rgb(255, 255, 255)`;
  }
  return (
    <a
      href="https://github.com/vicanso/location-rs"
      style={{
        position: "absolute",
        padding: "15px 30px",
        right: 0,
        top: 0,
      }}
    >
      <svg
        height="32"
        viewBox="0 0 16 16"
        width="32"
        aria-hidden="true"
        style={{
          fill: color,
        }}
      >
        <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0 0 16 8c0-4.42-3.58-8-8-8z" />
      </svg>
    </a>
  );
};

interface LocationInfo {
  country: string;
  province: string;
  city: string;
}

const App: FC = () => {
  const [messageApi, contextHolder] = message.useMessage();

  const [loading, setLoading] = useState(false);
  const [locationInfo, setLocationInfo] = useState<LocationInfo>();

  const onSearch = async (value: string) => {
    if (loading) {
      return;
    }
    const ip = value.trim() || "0.0.0.0";
    setLoading(true);
    try {
      const { data } = await axios.get<LocationInfo>(`/api/ip-locations/${ip}`);
      setLocationInfo(data);
    } catch (err: any) {
      let message = err?.message as string;
      let axiosErr = err as AxiosError;
      if (axiosErr?.response?.data) {
        let data = axiosErr.response.data as {
          message: string;
        };
        message = data.message || "";
      }
      messageApi.error(message || "get ip location fail", 10);
      setLocationInfo({} as LocationInfo);
    } finally {
      setLoading(false);
    }
  };

  const getSearchView = () => {
    const btn = (
      <Button
        style={{
          width: "60px",
        }}
        loading={loading}
        type="primary"
        shape="circle"
        icon={<SearchOutlined />}
      />
    );
    return (
      <Search
        autoFocus={true}
        placeholder="input the ip address"
        allowClear
        enterButton={btn}
        size="large"
        onSearch={onSearch}
      />
    );
  };
  let headerClass = "header";
  if (isDarkMode()) {
    headerClass += " dark";
  }

  return (
    <ConfigProvider
      theme={{
        algorithm: isDarkMode() ? darkAlgorithm : defaultAlgorithm,
      }}
    >
      {contextHolder}
      <Layout>
        {getGithubIcon(isDarkMode())}
        <Header className={headerClass}>
          <div className="contentWrapper">
            <div className="logo">IP Location</div>
          </div>
        </Header>
        <div className="fixSearch">
          {getSearchView()}
          <Descriptions className="mtop30" title="Location Information:">
            <Descriptions.Item label="Country">
              {locationInfo?.country || "--"}
            </Descriptions.Item>
            <Descriptions.Item label="Province">
              {locationInfo?.province || "--"}
            </Descriptions.Item>
            <Descriptions.Item label="City">
              {locationInfo?.city || "--"}
            </Descriptions.Item>
          </Descriptions>
        </div>
      </Layout>
    </ConfigProvider>
  );
};

export default App;
