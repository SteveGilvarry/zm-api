import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsWhereUniqueInput } from './monitors-where-unique.input';
import { MonitorsCreateInput } from './monitors-create.input';
import { MonitorsUpdateInput } from './monitors-update.input';

@ArgsType()
export class UpsertOneMonitorsArgs {

    @Field(() => MonitorsWhereUniqueInput, {nullable:false})
    where!: MonitorsWhereUniqueInput;

    @Field(() => MonitorsCreateInput, {nullable:false})
    create!: MonitorsCreateInput;

    @Field(() => MonitorsUpdateInput, {nullable:false})
    update!: MonitorsUpdateInput;
}
