import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { MonitorstatusService } from './monitorstatus.service';
import { Monitor_StatusCreateInput} from '../@generated/prisma-nestjs-graphql/monitor-status/monitor-status-create.input';
import { Monitor_StatusUpdateInput} from '../@generated/prisma-nestjs-graphql/monitor-status/monitor-status-update.input';
import { Monitor_StatusWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/monitor-status/monitor-status-where-unique.input';

@Resolver('Monitorstatus')
export class MonitorstatusResolver {
  constructor(private readonly monitorstatusService: MonitorstatusService) {}

  @Mutation('createMonitorstatus')
  async create(
    @Args('createMonitorstatusInput') createMonitorstatusInput: Monitor_StatusCreateInput
  ) {
    const created = await this.monitorstatusService.create(createMonitorstatusInput);
  }

  @Query('monitorstatus')
  findAll() {
    return this.monitorstatusService.findAll();
  }

  @Query('monitorstatus')
  findOne(@Args('monitorid') MonitorId: number) {
    return this.monitorstatusService.findOne({MonitorId});
  }

  @Mutation('updateMonitorstatus')
  update(
    @Args('MonitorId') MonitorId: number,
    @Args('updateMonitorstatusInput') updateMonitorstatusInput: Monitor_StatusUpdateInput) {
    return this.monitorstatusService.update({MonitorId}, updateMonitorstatusInput);
  }

  @Mutation('removeMonitorstatus')
  remove(@Args('MonitorId') MonitorId: number) {
    return this.monitorstatusService.remove({ MonitorId });
  }
}
