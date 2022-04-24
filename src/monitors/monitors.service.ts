import { Injectable } from '@nestjs/common';
import { PrismaService } from '../../prisma/prisma.service';
import { MonitorsCreateInput } from '../@generated/prisma-nestjs-graphql/monitors/monitors-create.input';
import { MonitorsWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/monitors/monitors-where-unique.input';
import { MonitorsUpdateInput } from '../@generated/prisma-nestjs-graphql/monitors/monitors-update.input';

@Injectable()
export class MonitorsService {
  constructor(private prisma: PrismaService) {}

  create(createMonitorInput: MonitorsCreateInput) {
    return this.prisma.monitors.create({
      data: createMonitorInput,
    });
  }

  findAll() {
    return this.prisma.monitors.findMany();
  }

  findOne(monitorsWhereUniqueInput: MonitorsWhereUniqueInput) {
    return this.prisma.monitors.findUnique({
      where: monitorsWhereUniqueInput,
    });
  }

  update(
    monitorsWhereUniqueInput: MonitorsWhereUniqueInput,
    updateMonitorInput: { updateMonitorInput: MonitorsUpdateInput },
  ) {
    return this.prisma.monitors.update({
      where: monitorsWhereUniqueInput,
      data: updateMonitorInput,
    });
  }

  remove(id: number) {
    return `This action removes a #${id} monitor`;
  }
}
