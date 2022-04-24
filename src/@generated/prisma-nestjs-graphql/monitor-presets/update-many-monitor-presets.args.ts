import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsUpdateManyMutationInput } from './monitor-presets-update-many-mutation.input';
import { MonitorPresetsWhereInput } from './monitor-presets-where.input';

@ArgsType()
export class UpdateManyMonitorPresetsArgs {

    @Field(() => MonitorPresetsUpdateManyMutationInput, {nullable:false})
    data!: MonitorPresetsUpdateManyMutationInput;

    @Field(() => MonitorPresetsWhereInput, {nullable:true})
    where?: MonitorPresetsWhereInput;
}
