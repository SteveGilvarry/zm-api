import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { ControlpresetsService } from './controlpresets.service';
import { ControlPresetsCreateInput } from '../@generated/prisma-nestjs-graphql/control-presets/control-presets-create.input';
import { ControlPresetsUpdateInput } from '../@generated/prisma-nestjs-graphql/control-presets/control-presets-update.input';
import { ControlPresetsWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/control-presets/control-presets-where-unique.input';
import { ControlPresetsMonitorIdPresetCompoundUniqueInput} from '../@generated/prisma-nestjs-graphql/control-presets/control-presets-monitor-id-preset-compound-unique.input';

@Resolver('Controlpreset')
export class ControlpresetsResolver {
  constructor(private readonly controlpresetsService: ControlpresetsService) {}

  @Mutation('createControlpreset')
  async create(
    @Args('createControlpresetInput') createControlpresetInput: ControlPresetsCreateInput,
    ) {
    const created = await this.controlpresetsService.create(createControlpresetInput);
  }

  @Query('controlpresets')
  findAll() {
    return this.controlpresetsService.findAll();
  }

  @Query('controlpreset')
  findOne(
    @Args('monitorid') MonitorId: number,
    @Args( 'preset') Preset: number
          ) {
    return this.controlpresetsService.findOne( {
      MonitorId_Preset: { MonitorId, Preset}
      }
    );
  }

  @Mutation('updateControlpreset')
  update(
    @Args( 'monitorid') MonitorId: number,
    @Args( "preset") Preset: number,
    @Args('updateControlpresetInput') updateControlpresetInput: ControlPresetsUpdateInput) {
    return this.controlpresetsService.update( {
      MonitorId_Preset: { MonitorId, Preset}
    }, updateControlpresetInput);
  }

  @Mutation('removeControlpreset')
  remove(@Args('id') id: number) {
    return this.controlpresetsService.remove(id);
  }
}
