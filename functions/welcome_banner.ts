import path from 'path';
import type { User } from "discord.js";
import fs from 'fs';
import { ensure } from '..';

export async function get_welcome_banner(user: User, server_name: string): Promise<string> {
  const htmlPath = path.resolve('./assets/welcome.html');
  const cssPath = path.resolve('./assets/welcome.css');
  const imagePath = path.resolve('./cache/welcome.png')

  let html = fs.readFileSync(htmlPath, 'utf-8');
  let css = fs.readFileSync(cssPath, 'utf-8');

  const replaceCSS = {
  }

  css = css.replace(/\$\{(.*?)\}/g, (_, rep_name: string) => {
    if (rep_name.startsWith('RANDOM<') && rep_name.endsWith('>')) {
      const r_match = rep_name.match(/RANDOM<(-?\d+),(-?\d+)>/);
      
      if (r_match) {
        const num_1 = parseInt(r_match[1]!, 10);
        const num_2 = parseInt(r_match[2]!, 10);
        
        const min_val = Math.min(num_1, num_2);
        const max_val = Math.max(num_1, num_2);
        
        const random_value = Math.floor(Math.random() * (max_val - min_val + 1)) + min_val;
        return random_value.toString();
      }
    }

    const lookup_value = replaceCSS[rep_name as keyof typeof replaceCSS];
    return lookup_value?.toString() ?? '';
  });

  const left_flourish = "file://" + path.join(__dirname, '..', 'assets', 'left_flourish.svg');
  const right_flourish = "file://" + path.join(__dirname, '..', 'assets', 'right_flourish.svg');

  const replaceHTML = {
    "CSS": css,
    "SERVERNAME": server_name.toUpperCase(),
    "AVATAR": user.displayAvatarURL(),
    "USERNAME": user.displayName,
    // "LEFT_FLOURISH": left_flourish,
    // "RIGHT_FLOURISH": right_flourish,
  }

  html = html.replace(/\$\{(.*?)\}/g, (_, rep_name: string) => {
    if (rep_name.startsWith('RANDOM<') && rep_name.endsWith('>')) {
      const r_match = rep_name.match(/RANDOM<(-?\d+),(-?\d+)>/);
      
      if (r_match) {
        const num_1 = parseInt(r_match[1]!, 10);
        const num_2 = parseInt(r_match[2]!, 10);
        
        const min_val = Math.min(num_1, num_2);
        const max_val = Math.max(num_1, num_2);
        
        const random_value = Math.floor(Math.random() * (max_val - min_val + 1)) + min_val;
        return random_value.toString();
      }
    }

    const lookup_value = replaceHTML[rep_name as keyof typeof replaceHTML];
    return lookup_value?.toString() ?? '';
  });

  fs.writeFileSync(process.env.CACHE_PATH! + 'welcome.html', html);

  const browser = user.client.browser;

  const page = await browser.newPage();

  await page.setViewport({width: 512, height: 128})

  await page.goto('file://' + process.env.CACHE_PATH! + 'welcome.html')

  await page.evaluate(async () => {
    await document.fonts.ready;
  });

  await waitForFileDeletion(imagePath);

  await page.screenshot({
    path: imagePath,
    omitBackground: true,
    fullPage: false,
  });

  if (ensure(process.env.DEV_MODE, 'No DEV_MODE ENV') == 'false') {
    await page.close();
  }

  return imagePath;
}

async function waitForFileDeletion(filePath: string) {
  while (fs.existsSync(filePath)) {
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
}
