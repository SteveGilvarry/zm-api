import { Injectable } from '@nestjs/common';
import { MonitorPresetsCreateInput } from '../@generated/prisma-nestjs-graphql/monitor-presets/monitor-presets-create.input';
import { MonitorPresetsUpdateInput } from '../@generated/prisma-nestjs-graphql/monitor-presets/monitor-presets-update.input';
import { MonitorPresetsWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/monitor-presets/monitor-presets-where-unique.input';
import { PrismaService} from '../../prisma/prisma.service';

@Injectable()
export class MonitorpresetsService {
  constructor(private prisma: PrismaService) {}

  create(createMonitorpresetInput: MonitorPresetsCreateInput) {
    return this.prisma.monitorPresets.create({
      data: createMonitorpresetInput
    });
  }

  findAll() {
    return this.prisma.monitorPresets.findMany();
  }

  findOne(monitorPresetWhereUniqueInput: MonitorPresetsWhereUniqueInput) {
    return this.prisma.monitorPresets.findUnique({
      where: monitorPresetWhereUniqueInput
    });
  }

  update(
    monitorPresetWhereUniqueInput: MonitorPresetsWhereUniqueInput,
    updateMonitorpresetInput: MonitorPresetsUpdateInput
  ) {
    return this.prisma.monitorPresets.update({
      where: monitorPresetWhereUniqueInput,
      data: updateMonitorpresetInput
    });
  }

  remove(monitorPresetWhereUniqueInput: MonitorPresetsWhereUniqueInput) {
    return this.prisma.monitorPresets.delete({
      where: monitorPresetWhereUniqueInput
    });
  }
}
