export interface AppConfig {
  name: string;
  port: number;
}

export function createApp(config: AppConfig): void {
  console.log(config.name);
}

export const VERSION = "1.0.0";
