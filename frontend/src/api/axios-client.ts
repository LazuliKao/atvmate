import axios from 'axios';
import type { AxiosRequestConfig } from 'axios';

export const customInstance = axios.create({
  baseURL: 'http://127.0.0.1:8000',
  headers: {
    'Content-Type': 'application/json',
  },
});

export const customInstanceFn = (
  config: AxiosRequestConfig,
  extraConfig: AxiosRequestConfig = {}
) => {
  const axiosConfig: AxiosRequestConfig = {
    ...config,
    ...extraConfig,
  };
  return customInstance(axiosConfig);
};

export default customInstance;
