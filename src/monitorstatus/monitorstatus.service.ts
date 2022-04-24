import { Injectable } from '@nestjs/common';
import { Monitor_StatusCreateInput} from '../@generated/prisma-nestjs-graphql/monitor-status/monitor-status-create.input';
import { Monitor_StatusUpdateInput} from '../@generated/prisma-nestjs-graphql/monitor-status/monitor-status-update.input';
import { Monitor_StatusWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/monitor-status/monitor-status-where-unique.input';
import { PrismaService} from '../../prisma/prisma.service';

@Injectable()
export class MonitorstatusService {
  constructor(private prisma: PrismaService) {}

  create(createMonitorstatusInput: Monitor_StatusCreateInput) {
    return this.prisma.monitor_Status.create({ data: createMonitorstatusInput });
  }

  findAll() {
    return this.prisma.monitor_Status.findMany();
  }

  findOne(monitorstatusWhereUniqueInput: Monitor_StatusWhereUniqueInput) {
    return this.prisma.monitor_Status.findUnique({
      where: monitorstatusWhereUniqueInput
    });
  }

  update(
    monitorstatusWhereUniqueInput: Monitor_StatusWhereUniqueInput,
    updateMonitorstatusInput: Monitor_StatusUpdateInput
  ) {
    return this.prisma.monitor_Status.update({
      where: monitorstatusWhereUniqueInput,
      data: updateMonitorstatusInput,
    });
  }

  remove(monitorstatusWhereUniqueInput: Monitor_StatusWhereUniqueInput) {
    return this.prisma.monitor_Status.delete({
      where: monitorstatusWhereUniqueInput,
    });
  }
}
