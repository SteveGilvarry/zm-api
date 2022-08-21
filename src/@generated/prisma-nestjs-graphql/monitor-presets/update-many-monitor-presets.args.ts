import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsUpdateManyMutationInput } from './monitor-presets-update-many-mutation.input';
import { Type } from 'class-transformer';
import { MonitorPresetsWhereInput } from './monitor-presets-where.input';

@ArgsType()
export class UpdateManyMonitorPresetsArgs {

    @Field(() => MonitorPresetsUpdateManyMutationInput, {nullable:false})
    @Type(() => MonitorPresetsUpdateManyMutationInput)
    data!: MonitorPresetsUpdateManyMutationInput;

    @Field(() => MonitorPresetsWhereInput, {nullable:true})
    @Type(() => MonitorPresetsWhereInput)
    where?: MonitorPresetsWhereInput;
}
