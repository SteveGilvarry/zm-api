import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsWhereUniqueInput } from './monitors-where-unique.input';
import { Type } from 'class-transformer';
import { MonitorsCreateInput } from './monitors-create.input';
import { MonitorsUpdateInput } from './monitors-update.input';

@ArgsType()
export class UpsertOneMonitorsArgs {

    @Field(() => MonitorsWhereUniqueInput, {nullable:false})
    @Type(() => MonitorsWhereUniqueInput)
    where!: MonitorsWhereUniqueInput;

    @Field(() => MonitorsCreateInput, {nullable:false})
    @Type(() => MonitorsCreateInput)
    create!: MonitorsCreateInput;

    @Field(() => MonitorsUpdateInput, {nullable:false})
    @Type(() => MonitorsUpdateInput)
    update!: MonitorsUpdateInput;
}
