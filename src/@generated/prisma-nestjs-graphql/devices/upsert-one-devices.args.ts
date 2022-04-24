import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesWhereUniqueInput } from './devices-where-unique.input';
import { DevicesCreateInput } from './devices-create.input';
import { DevicesUpdateInput } from './devices-update.input';

@ArgsType()
export class UpsertOneDevicesArgs {

    @Field(() => DevicesWhereUniqueInput, {nullable:false})
    where!: DevicesWhereUniqueInput;

    @Field(() => DevicesCreateInput, {nullable:false})
    create!: DevicesCreateInput;

    @Field(() => DevicesUpdateInput, {nullable:false})
    update!: DevicesUpdateInput;
}
