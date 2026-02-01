module.exports = {
  api: {
    input: './openapi.json',
    output: {
      mode: 'tags-split',
      target: 'src/api/default.ts',
      schemas: 'src/api/model',
      client: 'react-query',
      httpClient: 'axios',
      baseUrl: 'http://127.0.0.1:8000',
      prettier: true,
    },
    hooks: {
      afterAllFilesWrite: (files) => {
        const fs = require('fs');
        
        files.forEach(file => {
          if (file.includes('default.ts')) {
            let content = fs.readFileSync(file, 'utf-8');
            
            content = content.replace(/import type \{ DeviceList.*?\} from '\.\/model';\n*/g, '');
            
            content = content.replace(/\s*responseType:\s*['"]blob['"]\s*,/g, '');
            content = content.replace(/\s*responseType:\s*['"]blob['"]\s*\}/g, '}');
            content = content.replace(/,\s*,/g, ',');
            
            content = content.replace(/Promise<AxiosResponse<Blob>>/g, 'Promise<AxiosResponse<unknown>>');
            
            content = content.replace(/postDevicesBody:\s*Blob/g, 'postDevicesBody: AddDeviceRequest');
            content = content.replace(/postDevicesIpInputTextBody:\s*Blob/g, 'postDevicesIpInputTextBody: TextInputRequest');
            
            content = content.replace(/{data:\s*Blob\}/g, '{data: AddDeviceRequest}');
            content = content.replace(/{ip:\s*string;data:\s*Blob\}/g, '{ip: string;data: TextInputRequest}');
            
            content = content.replace(/PostDevicesMutationBody\s*=\s*Blob/g, 'PostDevicesMutationBody = AddDeviceRequest');
            content = content.replace(/PostDevicesIpInputTextMutationBody\s*=\s*Blob/g, 'PostDevicesIpInputTextMutationBody = TextInputRequest');
            
            const axiosImportEnd = content.indexOf("} from 'axios';");
            if (axiosImportEnd > 0) {
              const beforeImport = content.substring(0, axiosImportEnd + "} from 'axios';".length);
              const afterImport = content.substring(axiosImportEnd + "} from 'axios';".length);
              const typeImport = "\n\nimport type { DeviceList, AddDeviceRequest, ApiResponse, TextInputRequest } from './model';\n";
              content = beforeImport + typeImport + afterImport;
            }
            
            fs.writeFileSync(file, content, 'utf-8');
          }
        });
      },
    },
    mock: false,
    mutator: {
      path: './src/api/axios-client.ts',
      name: 'customInstanceFn',
    },
  },
};
