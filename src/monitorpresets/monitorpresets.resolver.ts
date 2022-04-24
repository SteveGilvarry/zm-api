import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { MonitorpresetsService } from './monitorpresets.service';
import { MonitorPresetsCreateInput } from '../@generated/prisma-nestjs-graphql/monitor-presets/monitor-presets-create.input';
import { MonitorPresetsUpdateInput } from '../@generated/prisma-nestjs-graphql/monitor-presets/monitor-presets-update.input';
import { MonitorsWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/monitors/monitors-where-unique.input';

@Resolver('Monitorpreset')
export class MonitorpresetsResolver {
  constructor(private readonly monitorpresetsService: MonitorpresetsService) {}

  @Mutation('createMonitorpreset')
  async create(
    @Args('createMonitorpresetInput') createMonitorpresetInput: MonitorPresetsCreateInput
  ) {
    const created = await this.monitorpresetsService.create(
      createMonitorpresetInput
    );
  }

  @Query('monitorpresets')
  findAll() {
    return this.monitorpresetsService.findAll();
  }

  @Query('monitorpreset')
  findOne(@Args('id') Id: number) {
    return this.monitorpresetsService.findOne({Id} );
  }

  @Mutation('updateMonitorpreset')
  update(
    @Args('id') Id: number,
    @Args('updateMonitorpresetInput') updateMonitorpresetInput: MonitorPresetsUpdateInput) {
    return this.monitorpresetsService.update( { Id }, updateMonitorpresetInput);
  }

  @Mutation('removeMonitorpreset')
  remove(@Args('id') Id: number) {
    return this.monitorpresetsService.remove( { Id } );
  }
}
