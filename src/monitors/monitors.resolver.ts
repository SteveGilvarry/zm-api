import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { MonitorsService } from './monitors.service';
import { MonitorsUpdateInput } from '../@generated/prisma-nestjs-graphql/monitors/monitors-update.input';
import { MonitorsCreateInput } from '../@generated/prisma-nestjs-graphql/monitors/monitors-create.input';

@Resolver('Monitor')
export class MonitorsResolver {
  constructor(private readonly monitorsService: MonitorsService) {}

  @Mutation('createMonitor')
  async create(
    @Args('createMonitorInput') createMonitorInput: MonitorsCreateInput,
  ) {
    const created = await this.monitorsService.create(createMonitorInput);
  }

  @Query('monitors')
  findAll() {
    return this.monitorsService.findAll();
  }

  @Query('monitor')
  findOne(@Args('id') Id: number) {
    return this.monitorsService.findOne({ Id });
  }

  @Mutation('updateMonitor')
  update(
    @Args('id') Id: number,
    @Args('updateMonitorInput') updateMonitorInput: MonitorsUpdateInput,
  ) {
    return this.monitorsService.update({ Id }, { updateMonitorInput });
  }

  @Mutation('removeMonitor')
  remove(@Args('id') id: number) {
    return this.monitorsService.remove(id);
  }
}
