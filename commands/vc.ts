import { SlashCommandBuilder, ChatInputCommandInteraction, type Interaction, ButtonStyle, Client, TextDisplayBuilder, ActionRowBuilder, SectionBuilder, ButtonBuilder, type MessageActionRowComponentBuilder } from "discord.js";
import type { Command } from "../deploy";
import { ActionRow, Button, Page, ProgressBar, ProgressBarSize, Section, SelectMenu, SelectMenuType, TextDisplay, Window } from "../functions/ui_framework/ui";
import type { Document } from "mongodb";
import { make_vc } from "../functions/dyn_voice_channel";
import { render } from "../functions/chapter_picker";

export type VcSettings = {
  _id: string,
  private: boolean,
  limit: number,
  default: boolean,
  permitted: {
    users_id: string[],
    roles_id: string[],
  }
}

// export default {
//   data: new SlashCommandBuilder()
//     .setName('vc')
//     .setDescription('Configure a VC to make.'),
//   async execute(interaction: ChatInputCommandInteraction) {
//     const client = interaction.client;
//     const collection = client.db.collection<VcSettings>('vc_settings');
//
//     const default_document = {
//       _id: interaction.user.id,
//       private: false,
//       limit: 0,
//       permitted: {
//         users_id: [],
//         roles_id: [],
//       }
//     };
//
//     const result = await collection.findOneAndUpdate(
//       { _id: interaction.user.id },
//       { $setOnInsert: default_document },
//       { 
//         upsert: true,
//         returnDocument: 'after'
//       }
//     ) ?? default_document;
//
//     const private_button = new Button(
//       async (p, i, d) => {
//         const is_private = (await set_result(i, [{$set:{private:{$not:"$private"}}}]))?.private ?? false;
//         await (is_private ? private_page:public_page).update(0, i);
//       },
//       { style: ButtonStyle.Secondary, disabled: false, label: async (_, __, i) => (await get_result(i!, i!.user.id)).private ? "Private":"Public", emoji: async (_, __, i) => (await get_result(i!, i!.user.id)).private ? "<:lock:1516602115022258186>":"<:unlock:1516602137050878094>" },
//       null,
//     );
//
//     const add_user = new Button(
//       async (_, i) => {
//         await vc_add_access_page.update(0, i);
//       },
//       { style: ButtonStyle.Primary, label: "Add Access", emoji: "<:add_user:1518803305151598693>"},
//       null,
//     )
//
//     const user_selection_menu = new SelectMenu(
//       async (_, i) => {
//         await set_result(i, { $addToSet: { "permitted.users_id": { $each: [...i.users.keys() ] }}})
//         await set_syn_content(i, private_page);
//         await private_page.update(0, i);
//       },
//       { type: SelectMenuType.User, min: 0, max: 25, placeholder: "Add User", options: [] },
//       null
//     );
//
//     const role_selection_menu = new SelectMenu(
//       async (_, i) => {
//         await set_result(i, { $addToSet: { "permitted.roles_id": { $each: [...i.roles.keys() ] }}})
//         await set_syn_content(i, private_page);
//         await private_page.update(0, i);
//       },
//       { type: SelectMenuType.Role, min: 0, max: 25, placeholder: "Add Role", options: [] },
//       null
//     );
//
//     const previous_button = new Button(
//       async (p, i, d) => {await p.update(d, i);},
//       { style: ButtonStyle.Secondary, emoji: "<:previous_button:1499160154828963940>", disabled: async (p, d, _) => (await p.previous(d)) < 0},
//       async (p: Page, d: number) => p.previous(d),
//     );
//
//     const next_button = new Button(
//       async (p, i, d) => {await p.update(d, i);},
//       { style: ButtonStyle.Secondary, emoji: "<:next_button:1499159772258242600>", disabled: async (p, d, _) => (await p.next(d)) > p.dynamicElements.length},
//       async (p: Page, d: number) => p.next(d),
//     );
//
//     const button_1 = new Button(
//       async (p, i, d: number) => {
//         await p.update(Math.min(d, 99), i)
//       },
//       { style: ButtonStyle.Secondary, label: "1" },
//       async (_: any, d: number, __: any): Promise<number> => {
//         return ((d ?? 0) * 10) + 1;
//       }
//     )
//     const button_2 = new Button(
//       async (p, i, d: number) => {
//         await p.update(Math.min(d, 99), i)
//       },
//       { style: ButtonStyle.Secondary, label: "2" },
//       async (_: any, d: number, __: any): Promise<number> => {
//         return ((d ?? 0) * 10) + 2;
//       }
//     )
//     const button_3 = new Button(
//       async (p, i, d: number) => {
//         await p.update(Math.min(d, 99), i)
//       },
//       { style: ButtonStyle.Secondary, label: "3" },
//       async (_: any, d: number, __: any): Promise<number> => {
//         return ((d ?? 0) * 10) + 3;
//       }
//     )
//     const button_4 = new Button(
//       async (p, i, d: number) => {
//         await p.update(Math.min(d, 99), i)
//       },
//       { style: ButtonStyle.Secondary, label: "4" },
//       async (_: any, d: number, __: any): Promise<number> => {
//         return ((d ?? 0) * 10) + 4;
//       }
//     )
//     const button_5 = new Button(
//       async (p, i, d: number) => {
//         await p.update(Math.min(d, 99), i)
//       },
//       { style: ButtonStyle.Secondary, label: "5" },
//       async (_: any, d: number, __: any): Promise<number> => {
//         return ((d ?? 0) * 10) + 5;
//       }
//     )
//     const button_6 = new Button(
//       async (p, i, d: number) => {
//         await p.update(Math.min(d, 99), i)
//       },
//       { style: ButtonStyle.Secondary, label: "6" },
//       async (_: any, d: number, __: any): Promise<number> => {
//         return ((d ?? 0) * 10) + 6;
//       }
//     )
//     const button_7 = new Button(
//       async (p, i, d: number) => {
//         await p.update(Math.min(d, 99), i)
//       },
//       { style: ButtonStyle.Secondary, label: "7" },
//       async (_: any, d: number, __: any): Promise<number> => {
//         return ((d ?? 0) * 10) + 7;
//       }
//     )
//     const button_8 = new Button(
//       async (p, i, d: number) => {
//         await p.update(Math.min(d, 99), i)
//       },
//       { style: ButtonStyle.Secondary, label: "8" },
//       async (_: any, d: number, __: any): Promise<number> => {
//         return ((d ?? 0) * 10) + 8;
//       }
//     )
//     const button_9 = new Button(
//       async (p, i, d: number) => {
//         await p.update(Math.min(d, 99), i)
//       },
//       { style: ButtonStyle.Secondary, label: "9" },
//       async (_: any, d: number, __: any): Promise<number> => {
//         return ((d ?? 0) * 10) + 9;
//       }
//     )
//     const button_0 = new Button(
//       async (p, i, d: number) => {
//         await p.update(Math.min(d, 99), i)
//       },
//       { style: ButtonStyle.Secondary, label: "0" },
//       async (_: any, d: number, __: any): Promise<number> => {
//         return ((d ?? 0) * 10) + 0;
//       }
//     )
//     const button_back_space = new Button(
//       async (p, i, d: {d: number}) => {
//         await p.update(Math.min(d.d, 99), i)
//       },
//       { style: ButtonStyle.Danger, emoji: "<:back_space:1520541439698665603>" },
//       async (_: any, d: number, __: any): Promise<{d: number}> => {
//         return {d: Math.floor((d ?? 0) / 10)};
//       }
//     )
//     const submit_button = new Button(
//       async (_p, i, d) => {
//         const settings = await set_result(i, { $set: { limit: d }});
//         const page = settings?.private ? private_page:public_page;
//         page.update(0, i);
//       },
//       { style: ButtonStyle.Success, emoji: "<:check_mark:1520519880279855144>"},
//       async (_: any, d: number, __: any) => {
//         return d;
//       }
//     )
//
//     const number_display = new TextDisplay(
//       async (_p, d, _i) => "# " + (d == 0 ? '∞':d.toString()) + "\n-# Max: 99"
//     );
//
//     const limit_progress_bar = new ProgressBar({ value: async (_,__,i) => (await get_result(i!, i!.user.id)).limit, max: 99, width: 5, size: ProgressBarSize.Moderate, pretext: async (_,__,i) => {const limit = (await get_result(i!, i!.user.id)).limit; return "Limit: " + (limit == 0 ? '∞': limit) + " "}})
//
//     const make_vc_button = new Button(
//       async (_,i,__) => {
//         if (!i.guild) return;
//         const channels = await i.guild.channels.fetch();
//         if (!channels) return;
//
//         const ids = JSON.parse(process.env.DYNAMIC_VOICE_CHANNELS ?? "[]");
//         const valid_ids = ids.filter((id: string) => channels.has(id));
//
//         if (valid_ids.length === 0) return
//
//         const vc = await make_vc(i.guild!, valid_ids[0], i.user.id);
//         if (!vc) return;
//         const invite: string = (await vc.createInvite({ maxAge: 36000, maxUses: 1, unique: true})).url
//         const action_row = new ActionRowBuilder<MessageActionRowComponentBuilder>().addComponents(new ButtonBuilder().setURL(invite).setStyle(ButtonStyle.Link).setLabel("Join"));
//         await i.update({ components: [action_row]})
//       },
//       { style: ButtonStyle.Success, label: "Make VC" },
//       null,
//     )
//
//     const vc_add_access_page = new Page("vc_add_access", client, true, true)
//       .addStaticElement(new TextDisplay("# Add User or Role:"))
//       .addStaticElement(new ActionRow([user_selection_menu]))
//       .addStaticElement(new ActionRow([role_selection_menu]));
//
//     const vc_input_number = new Page("vc_input_number", client, true, true)
//       .addStaticElement(number_display)
//       .addStaticElement(new ActionRow([button_1, button_2, button_3]))
//       .addStaticElement(new ActionRow([button_4, button_5, button_6]))
//       .addStaticElement(new ActionRow([button_7, button_8, button_9]))
//       .addStaticElement(new ActionRow([button_back_space, button_0, submit_button]));
//
//     const private_page = new Page("vc_priv", client, true, true, [], [], 5)
//       .addStaticElement(new TextDisplay("# VC Config Settings"))
//       .addStaticElement(new Section("### Private VC:", private_button))
//       .addStaticElement(new Section(limit_progress_bar, new Button(async (_p,i,_d) => {await vc_input_number.update(0, i);}, {style: ButtonStyle.Secondary, label: "Edit"}, null)))
//       .addStaticElement(new TextDisplay("### Allowed Access:"))
//       .addStaticElement(new Window())
//       .addStaticElement(new ActionRow([add_user, previous_button, next_button]))
//       .addStaticElement(new ActionRow([make_vc_button]));
//
//     await set_syn_content(interaction, private_page);
//
//     const public_page = new Page("vc_pub", client, true, true)
//       .addStaticElement(new TextDisplay("# VC Config Settings"))
//       .addStaticElement(new Section("### Public VC:", private_button))
//       .addStaticElement(new Section(limit_progress_bar, new Button(async (_p,i,_d) => {await vc_input_number.update(0, i);}, {style: ButtonStyle.Secondary, label: "Edit"}, null)))
//       .addStaticElement(new ActionRow([make_vc_button]));
//
//     await interaction.reply(await (result.private ? private_page:public_page).render(0, interaction))
//   },
// } as Command;



export default {
  data: new SlashCommandBuilder()
    .setName('vc')
    .setDescription('Configure a VC to make.'),
  async execute(interaction: ChatInputCommandInteraction) {
    const client = interaction.client;
    const db = client.db.collection<VcSettings>("vc_settings");

    const toggle_privacy_button = new Button(
      async (p, i, d: number) => {
        await set_result(i, { $set: { private: !p.globalData.private}});
        await p.update(d, i);
      },
      { 
        style: ButtonStyle.Secondary,
        emoji: async (p, _index, _i) => {return (p.globalData as VcSettings).private ? "<:lock:1516602115022258186>":"<:unlock:1516602137050878094>"},
        label: async (p, _index, _i) => {return (p.globalData as VcSettings).private ? "Private":"Public"}
      },
      async (_: any, index: number, __: any) => {return index},
    );

    const toggle_default_button = new Button(
      async (p, i, d: number) => {
        await set_result(i, { $set: { default: !p.globalData.default } });
        await p.update(d, i);
      },
      {
        style: async (p, _, __) => { return (p.globalData as VcSettings).default ? ButtonStyle.Primary:ButtonStyle.Secondary },
        emoji: async (p, _, __) => { return (p.globalData as VcSettings).default ? "<:check_mark:1520519880279855144>":"<:x_mark:1520519887506772119>" },
      },
      async (_:any, index: number, __:any) => {return index},
    )

    const private_page = new Page("vc_priv", client)
      .setIsEphemeral(true)
      .setIsContainer(true)
      .setWindowMax(5)
      .setPreRenderFunc(async (page, _, i) => {// Before rendering get the ensure the users settings and give them to the page.
        const vc_settings = await get_result(interaction || client, i?.user.id ?? interaction.user.id);
        page.globalData = vc_settings;
      })
      .addStaticElement(new TextDisplay("# VC Settings"))
      .addStaticElement(new Section("### Privacy:\n-# Control who can join your voice channel.", toggle_privacy_button))
      .addStaticElement(new Section("### Default:\n-# The default when makeing a VC via join making.", toggle_default_button));

    interaction.reply(await private_page.render(0, interaction));
  }
} as Command;

export async function get_result(interaction: Interaction | Client, user_id: string) {
  const collection = (interaction instanceof Client ? interaction:interaction.client).db.collection<VcSettings>('vc_settings');

  const default_document = {
    _id: user_id,
    private: false,
    limit: 0,
    default: false,
    permitted: {
      users_id: [],
      roles_id: [],
    }
  };
  
  const result = await collection.findOneAndUpdate(
    { _id: user_id },
    { $setOnInsert: default_document },
    { 
      upsert: true,
      returnDocument: 'after'
    }
  ) ?? default_document;

  return result;
}

async function set_result(interaction: Interaction, obj: Document) {
  const collection = interaction.client.db.collection<VcSettings>('vc_settings');

  return await collection.findOneAndUpdate({ _id: interaction.user.id }, obj, { returnDocument: 'after' });
}

async function set_syn_content(interaction: Interaction, page: Page) {
  const users = (await get_result(interaction, interaction.user.id)).permitted.users_id ?? [];
  const roles = (await get_result(interaction, interaction.user.id)).permitted.roles_id ?? [];

  const user_entry = users.map((id) => new Section(`<@${id}>`, new Button(
    async (p, i, d) => {
      await set_result(i, { $pull: { "permitted.users_id": d }})
      await set_syn_content(i, p);
      await p.update(0, i);
    },
    { style: ButtonStyle.Danger, emoji: "<:remove_user:1518803296041566293>" },
    id
  )))
  const roles_entry = roles.map((id) => new Section(`<@&${id}>`, new Button(
    async (p, i, d) => {
      await set_result(i, { $pull: { "permitted.roles_id": d }})
      await set_syn_content(i, p);
      await p.update(0, i);
    },
    { style: ButtonStyle.Danger, emoji: "<:remove_user:1518803296041566293>" },
    id
  )))

  page.setDynamicElement([...user_entry, ...roles_entry]);
}
