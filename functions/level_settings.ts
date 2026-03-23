import type { LevelSettings } from "../level";
import { ensure } from "..";
import { AttachmentBuilder, StringSelectMenuBuilder, StringSelectMenuOptionBuilder, ContainerBuilder, ButtonBuilder, MediaGalleryBuilder, MediaGalleryItemBuilder, ButtonStyle, type Client } from "discord.js";
import { getLevelBanner } from "../level";

export async function generateComponents(level_setting: LevelSettings, client: Client) {
  const cache_channel = ensure(await client.channels.fetch('1485110171922337812'), "Cant find cache_channel");
  if (!cache_channel.isTextBased()) {throw "Could not find text based cache_channel"};
  if (!cache_channel.isSendable()) {throw "Cant send in this Channel"};

  let user_settings: LevelSettings = level_setting
  console.log(user_settings);

  const user = ensure(await client.users.fetch(level_setting.user_id));

  const image_path = await getLevelBanner(user, user_settings);

  const image_atachment = new AttachmentBuilder(image_path);

  const message_cache = await cache_channel.send({ files: [image_atachment]});

  const image_url = message_cache.attachments.first()!.url;

  const level_settings_size_select = new StringSelectMenuBuilder()
    .setCustomId("level_settings_size")
    .setPlaceholder(`Change Size (${user_settings.is_large ?? 'Large', 'Small'})`)
    .addOptions(
      new StringSelectMenuOptionBuilder()
        .setLabel("Large")
        .setValue("large"),
      new StringSelectMenuOptionBuilder()
        .setLabel("Default Small")
        .setValue("small")
  );

  const level_settings_container = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addTextDisplayComponents((td) => td.setContent('# Level Banner Settings'))

  const primary_color_container = new ContainerBuilder()
    .setAccentColor(user_settings.primary_color)
    .addTextDisplayComponents((td) => td.setContent('**Primery Color:**\n-# This is the Color of the Bar'))
    .addActionRowComponents((ar) => ar.setComponents(new ButtonBuilder().setCustomId('level_settings_set_primery_color').setLabel('Set Color').setStyle(ButtonStyle.Secondary)));

  const secondary_color_container = new ContainerBuilder()
    .setAccentColor(user_settings.secondary_color)
    .addTextDisplayComponents((td) => td.setContent('**Secondary Color:**\n-# This is the Color of the Bar\'s backgrond'))
    .addActionRowComponents((ar) => ar.setComponents(new ButtonBuilder().setCustomId('level_settings_set_secondary_color').setLabel('Set Color').setStyle(ButtonStyle.Secondary)));

  const text_color_container = new ContainerBuilder()
    .setAccentColor(user_settings.text_color)
    .addTextDisplayComponents((td) => td.setContent('**Text Color:**\n-# This is the Color of the Text'))
    .addActionRowComponents((ar) => ar.setComponents(new ButtonBuilder().setCustomId('level_settings_set_text_color').setLabel('Set Color').setStyle(ButtonStyle.Secondary)));
  
  const backgrond_image_container = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addTextDisplayComponents((td) => td.setContent('**Backgrond Image:**\n-# This is the image that will be the backgrond'))
    .addActionRowComponents((ar) => ar.setComponents(new ButtonBuilder().setCustomId('level_settings_background').setLabel('Set Backgrond Image').setStyle(ButtonStyle.Secondary)));

  const size_container = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addTextDisplayComponents((td) => td.setContent('**Banner Size:**\n-# You can set the size of the Banner to Small or Large'))
    .addActionRowComponents((ar) => ar.setComponents(level_settings_size_select));

  const preview_container = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addTextDisplayComponents((td) => td.setContent('**Preciew:**\n-# This what the Banner will look like.'))
    .addMediaGalleryComponents(new MediaGalleryBuilder().addItems(new MediaGalleryItemBuilder().setURL(image_url).setDescription("Image 1")));

  return [
    level_settings_container,
    primary_color_container,
    secondary_color_container,
    text_color_container,
    backgrond_image_container,
    size_container,
    preview_container,
  ]
}
