import { Injectable } from '@nestjs/common';
import { GroupsCreateInput} from '../@generated/prisma-nestjs-graphql/groups/groups-create.input';
import { GroupsUpdateInput} from '../@generated/prisma-nestjs-graphql/groups/groups-update.input';
import { GroupsWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/groups/groups-where-unique.input';
import { PrismaService} from '../../prisma/prisma.service';

@Injectable()
export class GroupsService {
  constructor(private prisma: PrismaService) {}

  create(createGroupInput: GroupsCreateInput) {
    return this.prisma.groups.create({
      data: createGroupInput
    });
  }

  findAll() {
    return this.prisma.groups.findMany();
  }

  findOne(groupsWhereUniqueInput: GroupsWhereUniqueInput) {
    return this.prisma.groups.findUnique({
      where: groupsWhereUniqueInput
    });
  }

  update(
    groupsWhereUniqueInput: GroupsWhereUniqueInput,
    updateGroupInput: GroupsUpdateInput
  ) {
    return this.prisma.groups.update({
      where: groupsWhereUniqueInput,
      data: updateGroupInput
    });
  }

  remove(groupsWhereUniqueInput: GroupsWhereUniqueInput) {
    return this.prisma.groups.delete({
      where: groupsWhereUniqueInput
    });
  }
}
