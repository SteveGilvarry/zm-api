import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesUpdateInput } from './devices-update.input';
import { DevicesWhereUniqueInput } from './devices-where-unique.input';

@ArgsType()
export class UpdateOneDevicesArgs {

    @Field(() => DevicesUpdateInput, {nullable:false})
    data!: DevicesUpdateInput;

    @Field(() => DevicesWhereUniqueInput, {nullable:false})
    where!: DevicesWhereUniqueInput;
}
