import { PrismaClient } from "@prisma/client";

const prisma = new PrismaClient();

console.log("Prisma loaded");

// A `main` function so that you can use async/await
async function main() {
  // ... you will write your Prisma Client queries here
  const user = await prisma.user.create({
    data: { email: "email@email.com", name: "Test" },
  });

  const post = await prisma.post.create({
    data: {
      title: "My Post",
      content: "My post content",
      author: { connect: { id: user.id } },
    },
  });

  console.log({ post });

  await prisma.user.deleteMany({});
  await prisma.post.deleteMany({});
}

main()
  .catch((e) => {
    throw e;
  })
  .finally(async () => {
    await prisma.$disconnect();
  });
