import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsWhereInput } from './monitor-presets-where.input';
import { Type } from 'class-transformer';
import { MonitorPresetsOrderByWithRelationInput } from './monitor-presets-order-by-with-relation.input';
import { MonitorPresetsWhereUniqueInput } from './monitor-presets-where-unique.input';
import { Int } from '@nestjs/graphql';
import { MonitorPresetsScalarFieldEnum } from './monitor-presets-scalar-field.enum';

@ArgsType()
export class FindManyMonitorPresetsArgs {

    @Field(() => MonitorPresetsWhereInput, {nullable:true})
    @Type(() => MonitorPresetsWhereInput)
    where?: MonitorPresetsWhereInput;

    @Field(() => [MonitorPresetsOrderByWithRelationInput], {nullable:true})
    @Type(() => MonitorPresetsOrderByWithRelationInput)
    orderBy?: Array<MonitorPresetsOrderByWithRelationInput>;

    @Field(() => MonitorPresetsWhereUniqueInput, {nullable:true})
    @Type(() => MonitorPresetsWhereUniqueInput)
    cursor?: MonitorPresetsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [MonitorPresetsScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof MonitorPresetsScalarFieldEnum>;
}
