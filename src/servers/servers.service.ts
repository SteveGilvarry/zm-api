import { Injectable } from '@nestjs/common';
import { ServersCreateInput} from '../@generated/prisma-nestjs-graphql/servers/servers-create.input';
import { ServersUpdateInput} from '../@generated/prisma-nestjs-graphql/servers/servers-update.input';
import { ServersWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/servers/servers-where-unique.input';
import { PrismaService} from '../../prisma/prisma.service';

@Injectable()
export class ServersService {
  constructor(private prisma: PrismaService) {}

  create(createServerInput: ServersCreateInput) {
    return this.prisma.servers.create({ data: createServerInput });
  }

  findAll() {
    return this.prisma.servers.findMany();
  }

  findOne(serversWhereUniqueInput: ServersWhereUniqueInput) {
    return this.prisma.servers.findUnique({
      where: serversWhereUniqueInput
    });
  }

  update(
    serversWhereUniqueInput: ServersWhereUniqueInput,
    updateServerInput: ServersUpdateInput
  ) {
    return this.prisma.servers.update({
      where: serversWhereUniqueInput,
      data: updateServerInput
    });
  }

  remove(serversWhereUniqueInput: ServersWhereUniqueInput) {
    return this.prisma.servers.delete({
      where: serversWhereUniqueInput
    });
  }
}
