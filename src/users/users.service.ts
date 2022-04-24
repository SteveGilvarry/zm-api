import { Injectable } from '@nestjs/common';
import { PrismaService } from '../../prisma/prisma.service';
import { UsersCreateInput } from '../@generated/prisma-nestjs-graphql/users/users-create.input';
import { UsersUpdateInput } from '../@generated/prisma-nestjs-graphql/users/users-update.input';
import { UsersWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/users/users-where-unique.input';

@Injectable()
export class UsersService {
  constructor(private prisma: PrismaService) {}

  create(createUserInput: UsersCreateInput) {
    return 'This action adds a new user';
  }

  findAll() {
    return this.prisma.users.findMany();
  }

  findOne(userWhereUniqueInput: UsersWhereUniqueInput) {
    return this.prisma.users.findUnique({
      where: userWhereUniqueInput,
    });
  }

  update(
    usersWhereUniqueInput: UsersWhereUniqueInput,
    updateUserInput: UsersUpdateInput,
  ) {
    return this.prisma.users.update({
      where: usersWhereUniqueInput,
      data: updateUserInput,
    });
  }

  remove(id: number) {
    return `This action removes a #${id} user`;
  }
}
