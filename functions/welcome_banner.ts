import { resolve } from 'path';
import type { User } from "discord.js";
import fs from 'fs';
import { ensure } from '..';

export async function get_welcome_banner(user: User): Promise<string> {
  const htmlPath = '../assets/welcome.html';
  const cssPath = '../assets/welcome.css';
  const imagePath = resolve('../cache/welcome.png')

  let html = fs.readFileSync(htmlPath, 'utf-8');
  let css = fs.readFileSync(cssPath, 'utf-8');

  const replaceCSS = {
  }

  css = css.replace(/\$\{(.*?)\}/g, (_, repName) => {
    const value = replaceCSS[repName as keyof typeof replaceCSS];
    return value?.toString() ?? '';
  });

  const replaceHTML = {
  }

  html = html.replace(/\$\{(.*?)\}/g, (_, repName) => {
    const value = replaceHTML[repName as keyof typeof replaceHTML];
    return value?.toString() ?? '';
  });

  fs.writeFileSync(process.env.CACHE_PATH! + 'welcome.html', html);

  const browser = user.client.browser;

  const page = await browser.newPage();

  await page.setViewport({width: 512, height: 128})

  await page.goto('file://' + process.env.CACHE_PATH! + 'html.html')

  await page.evaluate(async () => {
    await document.fonts.ready;
  });

  await waitForFileDeletion(imagePath);

  await page.screenshot({
    path: imagePath,
    omitBackground: true,
    fullPage: true,
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
