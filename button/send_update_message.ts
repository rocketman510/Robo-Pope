import { ButtonInteraction, EmbedBuilder, MessageFlags } from "discord.js";
import type { Button } from "../deploy";
import { ensure } from "..";

export default {
    data: "send_update_message",
    async execute(interaction: ButtonInteraction) {
      const client = interaction.client;
      const announcement_channel_ids = JSON.parse(ensure(process.env.ANNOUNCEMENT_CHANNAL_IDS, "No ANNOUNCEMENT_CHANNAL_IDS ENV"))

      const embed = new EmbedBuilder()
        .setColor(0x29292D)
        .setTitle(' ')
        .setDescription(`# Update v0.2.0 - Bundle of Banners Update
Dear,\n@everyone\nWelcome one and all to all new Robo-Pope version 0.2.0. In this update much has happened and much is still to come. I Robo-Pope have cooked up a delicious consommé of features for you my lovely brethren and *cough cough clare cough* to enjoy. Now lets not kid ourself and get right into the action!\n## 1. Leaderboard\nI have added a leaderboard so that all of you can compete for getting no maidens. The leaderboard will show the top 10 biggest yappers in the server; and it updates Live! The leaderboard is in the <#1484688796342550569> channel.\n## 2. Sharing your Level Banner\nI have made it so that you can share you Level Banner. If you run \`/level\` you will see that under the banner there are 2 new buttons. The first of those buttons is a share button and it will enable you to post your Level Banner in the chat you are talking in.\n## 3. One Word Story automatic enforcement.\nI made it so that <#1486807946246492260> will now be automatically enforced by me.\n## 4. Level System Rebalance\nThe old level system was to hard for people to level up late into yapping and to easy for people that just started. I rebalanced the level system. So yes your level will now be different. :pensive:\n## 5. ALL NEW Level Banners\nI have added all new Level Banners and now you can customize your Level Banner! You will be able to change the following:\n- Primary color\n- Secondary color\n- Text color\n- Background image <:por1:1487227954898538496><:pro2:1487227945348239564>\n- Banner size <:por1:1487227954898538496><:pro2:1487227945348239564>\nUnfortunately the Background image and Banner size options are only for <@&1425570661165961369>, <@&1483145047716266084>,<@&1425988510896095282>, and <@&1484252302704509038>. You can change your banners look by clicking the second button under the Level Banner.\n\nIf you want to see my source code then you can go the the [github](https://github.com/rocketman510/Robo-Pope)\nYours truly,\nRobo-Pope`)

      for (const id of announcement_channel_ids) {
        const channel = await client.channels.fetch(id);
        if (!channel?.isSendable()) return;
        channel.send({ embeds: [embed] })
      }
    },
} as Button;
