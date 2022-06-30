import { Callout } from "../../components";

export const callout = {
  render: Callout,
  description: "Display the enclosed content in a callout box",
  children: ["paragraph", "tag", "list"],
  attributes: {
    title: {
      type: String,
      description: "The title displayed at the top of the callout",
    },
  },
};
