import { ContainerBuilder, MessageFlags, TextDisplayBuilder } from "discord.js";
import type { ButtonBuilder, Interaction, MessageReplyOptions } from "discord.js";

type Element = TextDisplay

export class Page {
  public customID: string;
  public isContainer: boolean;
  readonly staticElements: Element[];
  readonly dynamicElements: Element[];
  readonly cache = new Map<string, Element>;
  readonly data = new Map<string, any>;

  constructor(custom_id: string, is_container?: boolean, static_elements?: Element[], dynamic_elements?: Element[]) {
    this.customID = custom_id;
    this.isContainer = is_container ?? false;
    this.staticElements = static_elements ?? [];
    this.dynamicElements = dynamic_elements ?? [];

    for (const element of this.staticElements) {
      element.page = this;
    }
    for (const element of this.dynamicElements) {
      element.page = this;
    }
  }

  public addStaticElements(element: Element): Page {
    element.page = this;
    this.staticElements.push(element);
    return this;
  }

  public render(): MessageReplyOptions {
    if (this.isContainer) {
      const container = new ContainerBuilder()

      for (const element of this.staticElements) {
        element.apply(container);
      }

      return { components: [container], flags: MessageFlags.IsComponentsV2 }
    } else {
      let components = [];

      for (const element of this.staticElements) {
        components.push(element.builder);
      }

      return { components, flags: MessageFlags.IsComponentsV2 }
    }
  }
}

type ButtonExecution = (interaction: Interaction) => void;

export class Button {
  public builder: ButtonBuilder;
  public execution: ButtonExecution;
  public page!: Page;

  constructor(execution: ButtonExecution, builder: ButtonBuilder) {
    this.builder = builder;
    this.execution = execution;
  }
}

export class TextDisplay {
  public builder: TextDisplayBuilder;
  public page!: Page;

  constructor(builder: TextDisplayBuilder | string) {
    if (typeof builder === "string") {
      this.builder = new TextDisplayBuilder().setContent(builder);
    } else {
      this.builder = builder;
    } 
  }

  public apply(container: ContainerBuilder) {
    container.addTextDisplayComponents(this.builder);
  }
}
