import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesWhereUniqueInput } from './devices-where-unique.input';

@ArgsType()
export class DeleteOneDevicesArgs {

    @Field(() => DevicesWhereUniqueInput, {nullable:false})
    where!: DevicesWhereUniqueInput;
}
