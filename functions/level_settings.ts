import type { LevelSettings } from "../level";
import { ensure } from "..";
import { AttachmentBuilder, StringSelectMenuBuilder, StringSelectMenuOptionBuilder, ContainerBuilder, ButtonBuilder, MediaGalleryBuilder, MediaGalleryItemBuilder, ButtonStyle, type Client } from "discord.js";
import { getLevelBanner } from "../level";
import fs from "fs";

export async function generateComponents(level_setting: LevelSettings, client: Client) {
  const cache_channel = ensure(await client.channels.fetch('1485110171922337812'), "Can't find cache_channel");
  if (!cache_channel.isTextBased()) throw "Could not find text based cache_channel";
  if (!cache_channel.isSendable()) throw "Can't send in this Channel";


  const guild = await client.guilds.fetch(level_setting.guild_id!);
  const guild_member = await guild.members.fetch(level_setting.user_id);
  const pro_roles = JSON.parse(ensure(process.env.LEVEL_PRO_SETTINGS_ROLES, "No LEVEL_PRO_SETTINGS_ROLES ENV"));
  const is_pro = guild_member.roles.cache.hasAny(...pro_roles);

  let user_settings: LevelSettings = level_setting

  const user = ensure(await client.users.fetch(level_setting.user_id));

  const image_path = await getLevelBanner(user, user_settings);

  const image_attachment = new AttachmentBuilder(image_path);

  const message_cache = await cache_channel.send({ files: [image_attachment]});

  if (fs.existsSync(image_path)) {
    fs.unlinkSync(image_path);
  }

  const image_url = message_cache.attachments.first()!.url;

  const level_settings_size_select = new StringSelectMenuBuilder()
    .setCustomId("level_settings_size")
    .setPlaceholder(`Change Size (${user_settings.is_large ? 'Large':'Small'})`)
    .addOptions(
      new StringSelectMenuOptionBuilder()
        .setLabel("Large")
        .setValue("large"),
      new StringSelectMenuOptionBuilder()
        .setLabel("Default Small")
        .setValue("small")
    )
    .setDisabled(!is_pro);

  const level_settings_frost_select = new StringSelectMenuBuilder()
    .setCustomId('level_settings_frost')
    .setPlaceholder(`Change Backdrop Style (${user_settings.frost?'Frost':'Clear'})`)
    .setOptions(
      new StringSelectMenuOptionBuilder().setLabel('Frost').setValue('frost'),
      new StringSelectMenuOptionBuilder().setLabel('Clear').setValue('clear')
    )
    .setDisabled(!is_pro);

  const level_settings_container = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addTextDisplayComponents((td) => td.setContent('# Level Banner Settings\nThis is the Settings page for configuring the look of your Level Banner. If you don\'t understand how to set the color, then check out our resource [here](<https://htmlcolorcodes.com/>).\nIf you are interested in unlocking **PRO** features consider **Boosting** the server.'))

  const primary_color_container = new ContainerBuilder()
    .setAccentColor(user_settings.primary_color)
    .addTextDisplayComponents((td) => td.setContent('**Primary Color:**\n-# This is the Color of the Bar.'))
    .addActionRowComponents((ar) => ar.setComponents(new ButtonBuilder().setCustomId('level_settings_set_primary_color').setLabel('Set Color').setStyle(ButtonStyle.Secondary)));

  const secondary_color_container = new ContainerBuilder()
    .setAccentColor(user_settings.secondary_color)
    .addTextDisplayComponents((td) => td.setContent('**Secondary Color:**\n-# This is the Color of the Bar\'s background'))
    .addActionRowComponents((ar) => ar.setComponents(new ButtonBuilder().setCustomId('level_settings_set_secondary_color').setLabel('Set Color').setStyle(ButtonStyle.Secondary)));

  const text_color_container = new ContainerBuilder()
    .setAccentColor(user_settings.text_color)
    .addTextDisplayComponents((td) => td.setContent('**Text Color:**\n-# This is the Color of the Text'))
    .addActionRowComponents((ar) => ar.setComponents(new ButtonBuilder().setCustomId('level_settings_set_text_color').setLabel('Set Color').setStyle(ButtonStyle.Secondary)));

  const frost_container = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addTextDisplayComponents((td) => td.setContent('**Frosted Background <:pro1:1487227954898538496><:pro2:1487227945348239564>:**\n-# This toggles backgrond blur for the progress bar'))
    .addActionRowComponents((ar) => ar.setComponents(level_settings_frost_select));

  const height = level_setting.is_large ? 125 : (level_setting.has_custom_background ? 8 : 0) + 22;
  const width = level_setting.is_large ? 512 : (level_setting.has_custom_background ? 8 : 0) + 256;

  const background_image_description = `**Background Image <:pro1:1487227954898538496><:pro2:1487227945348239564>:**\n-# This is the image that will be the background\nResolution: ${height + 'x' + width}`

  let background_image_container = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addTextDisplayComponents((td) => td.setContent(background_image_description))
    .addActionRowComponents((ar) => ar.setComponents(new ButtonBuilder().setCustomId('level_settings_background').setLabel('Set Background Image').setStyle(ButtonStyle.Secondary).setDisabled(!is_pro)))

  const size_container = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addTextDisplayComponents((td) => td.setContent('**Banner Size <:pro1:1487227954898538496><:pro2:1487227945348239564>:**\n-# You can set the size of the Banner to Small or Large'))
    .addActionRowComponents((ar) => ar.setComponents(level_settings_size_select));

  const preview_container = new ContainerBuilder()
    .setAccentColor(0x242429)
    .addTextDisplayComponents((td) => td.setContent('**Preview:**\n-# This what the Banner will look like.'))
    .addMediaGalleryComponents(new MediaGalleryBuilder().addItems(new MediaGalleryItemBuilder().setURL(image_url).setDescription("Image 1")));

  const warning_container = new ContainerBuilder()
    .setAccentColor(0xFFAA00)
    .addTextDisplayComponents((td) => td.setContent('### :warning: Refresh time can vary, be patient and don\'t try to use it between Refreshes'))

  return [
    level_settings_container,
    primary_color_container,
    secondary_color_container,
    text_color_container,
    background_image_container,
    size_container,
    frost_container,
    preview_container,
    warning_container,
  ]
}
